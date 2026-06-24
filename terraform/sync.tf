# B2A Sync Deposits Worker
#
# Modo polling HTTP (no WebSocket) para máxima confiabilidad en Lambda.
# Se ejecuta cada pocos minutos vía EventBridge.
# Fase 4 / Prod: configure RPC to mainnet.base.org via var; use appropriate schedule and max blocks for mainnet costs.
# Mantiene estado del último bloque en la tabla de balances (meta).
#
# Este worker es crítico para el modelo B2A: acredita los depósitos on-chain
# al saldo off-chain del agente de forma automática.

data "archive_file" "sync_zip" {
  type        = "zip"
  source_file = "../api/target/lambda/sync_deposits/bootstrap"
  output_path = "sync_payload.zip"
}

# Dedicated IAM Role for Sync (needs write to balances for deposits)
resource "aws_iam_role" "b2a_sync_lambda_role" {
  name = "b2a-sync-lambda-role-${var.environment}"

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

resource "aws_iam_role_policy" "b2a_sync_dynamodb_policy" {
  name = "B2ASyncDynamoDBAccess"
  role = aws_iam_role.b2a_sync_lambda_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "dynamodb:GetItem",
          "dynamodb:PutItem",
          "dynamodb:UpdateItem"
        ]
        Resource = [
          aws_dynamodb_table.b2a_balances.arn,
          aws_dynamodb_table.b2a_nonces.arn
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
        Action = [
          "secretsmanager:GetSecretValue"
        ]
        Resource = "arn:aws:secretsmanager:*:*:secret:b2a/*"
      }
    ]
  })
}

# Lambda Function for Sync Deposits
resource "aws_lambda_function" "b2a_sync" {
  function_name    = "b2a_sync_${var.environment}"
  role             = aws_iam_role.b2a_sync_lambda_role.arn
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  architectures    = ["arm64"]
  timeout          = 300 # Allow time for block range processing

  filename         = data.archive_file.sync_zip.output_path
  source_code_hash = data.archive_file.sync_zip.output_base64sha256

  environment {
    variables = {
      USE_DYNAMODB    = "true"
      BALANCES_TABLE  = aws_dynamodb_table.b2a_balances.name
      NONCES_TABLE    = aws_dynamodb_table.b2a_nonces.name
      RPC_URL         = var.rpc_url
      CONTRACT_ADDRESS= var.contract_address
    }
  }

  depends_on = [aws_iam_role_policy.b2a_sync_dynamodb_policy]
}

# EventBridge Rule: run sync every 2 minutes
resource "aws_cloudwatch_event_rule" "sync_cron" {
  name                = "b2a_sync_cron_${var.environment}"
  description         = "Trigger B2A deposit sync periodically (configurable via sync_schedule variable)"
  schedule_expression = var.sync_schedule
}

resource "aws_cloudwatch_event_target" "trigger_sync" {
  rule      = aws_cloudwatch_event_rule.sync_cron.name
  target_id = "TriggerSync"
  arn       = aws_lambda_function.b2a_sync.arn
}

resource "aws_lambda_permission" "allow_eventbridge_sync" {
  statement_id  = "AllowExecutionFromEventBridgeSync"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.b2a_sync.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.sync_cron.arn
}

# Output moved to outputs.tf for uniqueness