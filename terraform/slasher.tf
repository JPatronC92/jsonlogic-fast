# Slasher Lambda Package
data "archive_file" "slasher_zip" {
  type        = "zip"
  source_file = "../api/target/lambda/slasher/bootstrap"
  output_path = "slasher_payload.zip"
}

# Lambda Function (Slasher)
resource "aws_lambda_function" "b2a_slasher" {
  function_name    = "b2a_slasher_${var.environment}"
  role             = aws_iam_role.lambda_execution_role.arn
  handler          = "bootstrap" 
  runtime          = "provided.al2023"
  architectures    = ["arm64"]
  timeout          = 300 # Slasher needs more time (5 minutes) for blockchain interactions

  filename         = data.archive_file.slasher_zip.output_path
  source_code_hash = data.archive_file.slasher_zip.output_base64sha256

  environment {
    variables = {
      USE_DYNAMODB    = "true"
      BALANCES_TABLE  = aws_dynamodb_table.balances.name
      NONCES_TABLE    = aws_dynamodb_table.nonces.name
      RPC_URL         = var.rpc_url
      CONTRACT_ADDRESS= var.contract_address
      PRIVATE_KEY     = var.private_key
    }
  }
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
