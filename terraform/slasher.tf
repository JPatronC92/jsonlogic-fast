# Slasher Lambda Package
data "archive_file" "slasher_zip" {
  type        = "zip"
  source_file = "../api/target/lambda/slasher/bootstrap"
  output_path = "slasher_payload.zip"
}

# Load private key securely from Secrets Manager (not passed via TF var)
data "aws_secretsmanager_secret_version" "slasher_private_key" {
  secret_id = "b2a/slasher-private-key-${var.environment}"
}

# IAM Role for B2A Slasher Lambda (least privilege - read only on balances)
resource "aws_iam_role" "b2a_slasher_lambda_role" {
  name = "b2a-slasher-lambda-role-${var.environment}"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy" "b2a_slasher_dynamodb_policy" {
  name = "B2ASlasherDynamoDBAccess"
  role = aws_iam_role.b2a_slasher_lambda_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "dynamodb:GetItem",
          "dynamodb:Scan"
        ]
        # Solo balances (el slasher no toca nonces)
        Resource = [
          aws_dynamodb_table.b2a_balances.arn
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Resource = "arn:aws:logs:*:*:*"
      },
      {
        Effect = "Allow"
        Action = "secretsmanager:GetSecretValue"
        Resource = "arn:aws:secretsmanager:*:*:secret:b2a/slasher-private-key-${var.environment}-*"
      }
    ]
  })
}

# Lambda Function (Slasher)
resource "aws_lambda_function" "b2a_slasher" {
  function_name    = "b2a_slasher_${var.environment}"
  role             = aws_iam_role.b2a_slasher_lambda_role.arn
  handler          = "bootstrap" 
  runtime          = "provided.al2023"
  architectures    = ["arm64"]
  timeout          = 300 # Slasher needs more time (5 minutes) for blockchain interactions

  filename         = data.archive_file.slasher_zip.output_path
  source_code_hash = data.archive_file.slasher_zip.output_base64sha256

  environment {
    variables = {
      USE_DYNAMODB    = "true"
      BALANCES_TABLE  = aws_dynamodb_table.b2a_balances.name
      NONCES_TABLE    = aws_dynamodb_table.b2a_nonces.name
      RPC_URL         = var.rpc_url
      CONTRACT_ADDRESS= var.contract_address
      PRIVATE_KEY     = data.aws_secretsmanager_secret_version.slasher_private_key.secret_string
    }
  }

  depends_on = [aws_iam_role_policy.b2a_slasher_dynamodb_policy]
}

# EventBridge Rule to trigger Slasher every hour
resource "aws_cloudwatch_event_rule" "slasher_cron" {
  name                = "b2a_slasher_cron_${var.environment}"
  description         = "Trigger B2A Slasher periodically"
  schedule_expression = "rate(1 hour)"
}

resource "aws_cloudwatch_event_target" "trigger_slasher" {
  rule      = aws_cloudwatch_event_rule.slasher_cron.name
  target_id = "TriggerSlasher"
  arn       = aws_lambda_function.b2a_slasher.arn
}

resource "aws_lambda_permission" "allow_eventbridge_slasher" {
  statement_id  = "AllowExecutionFromEventBridge"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.b2a_slasher.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.slasher_cron.arn
}
