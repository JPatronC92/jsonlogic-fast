output "api_gateway_url" {
  description = "The URL of the API Gateway HTTP API"
  value       = aws_apigatewayv2_api.http_api.api_endpoint
}

output "api_lambda_arn" {
  description = "The ARN of the B2A API Lambda function"
  value       = aws_lambda_function.b2a_api.arn
}

output "slasher_lambda_arn" {
  description = "The ARN of the B2A Slasher Lambda function"
  value       = aws_lambda_function.b2a_slasher.arn
}

output "sync_lambda_arn" {
  description = "The ARN of the B2A Sync Deposits Lambda function"
  value       = aws_lambda_function.b2a_sync.arn
}

output "environment" {
  description = "The environment used for resource namespacing"
  value       = var.environment
}
