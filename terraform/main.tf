provider "aws" {
  region = var.aws_region
}

# DynamoDB Tables
resource "aws_dynamodb_table" "balances" {
  name           = "${var.balances_table_name}_${var.environment}"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "address"

  attribute {
    name = "address"
    type = "S"
  }

  tags = {
    Environment = var.environment
    Project     = "B2A"
  }
}

resource "aws_dynamodb_table" "nonces" {
  name           = "${var.nonces_table_name}_${var.environment}"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "id"

  attribute {
    name = "id"
    type = "S"
  }

  ttl {
    attribute_name = "ttl"
    enabled        = true
  }

  tags = {
    Environment = var.environment
    Project     = "B2A"
  }
}

# IAM Role for Lambda
resource "aws_iam_role" "lambda_execution_role" {
  name = "b2a_lambda_exec_role_${var.environment}"

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

# IAM Policy for DynamoDB Access
resource "aws_iam_policy" "lambda_dynamodb_policy" {
  name        = "b2a_dynamodb_policy_${var.environment}"
  description = "Allows Lambda to access DynamoDB tables"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "dynamodb:PutItem",
          "dynamodb:GetItem",
          "dynamodb:UpdateItem",
          "dynamodb:DeleteItem",
          "dynamodb:Scan"
        ]
        Effect   = "Allow"
        Resource = [
          aws_dynamodb_table.balances.arn,
          aws_dynamodb_table.nonces.arn
        ]
      },
      {
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Effect   = "Allow"
        Resource = "arn:aws:logs:*:*:*"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_policy_attach" {
  role       = aws_iam_role.lambda_execution_role.name
  policy_arn = aws_iam_policy.lambda_dynamodb_policy.arn
}

# Package the Lambda payload
data "archive_file" "lambda_zip" {
  type        = "zip"
  # This expects cargo-lambda output in the root of the project
  source_file = "../api/target/lambda/lambda/bootstrap"
  output_path = "lambda_payload.zip"
}

# Lambda Function (API)
resource "aws_lambda_function" "b2a_api" {
  function_name    = "b2a_api_${var.environment}"
  role             = aws_iam_role.lambda_execution_role.arn
  handler          = "bootstrap" # Rust custom runtime requires this handler name
  runtime          = "provided.al2023" # Amazon Linux 2023 for Rust
  architectures    = ["arm64"] # Or x86_64 depending on build
  timeout          = 30

  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256

  environment {
    variables = {
      USE_DYNAMODB    = "true"
      BALANCES_TABLE  = aws_dynamodb_table.balances.name
      NONCES_TABLE    = aws_dynamodb_table.nonces.name
      RPC_URL         = var.rpc_url
      CONTRACT_ADDRESS= var.contract_address
    }
  }

  depends_on = [aws_iam_role_policy_attachment.lambda_policy_attach]
}

# API Gateway HTTP API
resource "aws_apigatewayv2_api" "http_api" {
  name          = "b2a_http_api_${var.environment}"
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_stage" "default_stage" {
  api_id      = aws_apigatewayv2_api.http_api.id
  name        = "$default"
  auto_deploy = true
}

resource "aws_apigatewayv2_integration" "lambda_integration" {
  api_id                 = aws_apigatewayv2_api.http_api.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.b2a_api.invoke_arn
  integration_method     = "POST"
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "default_route" {
  api_id    = aws_apigatewayv2_api.http_api.id
  route_key = "$default"
  target    = "integrations/${aws_apigatewayv2_integration.lambda_integration.id}"
}

# Lambda Permission for API Gateway
resource "aws_lambda_permission" "apigw_invoke" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.b2a_api.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.http_api.execution_arn}/*/*"
}
