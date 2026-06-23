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
