# Basic CloudWatch Alarms and SNS for critical B2A components

resource "aws_sns_topic" "b2a_alerts" {
  name = "b2a-alerts-${var.environment}"
}

# Alarm for API Lambda errors
resource "aws_cloudwatch_metric_alarm" "api_lambda_errors" {
  alarm_name          = "b2a-api-errors-${var.environment}"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "1"
  metric_name         = "Errors"
  namespace           = "AWS/Lambda"
  period              = "60"
  statistic           = "Sum"
  threshold           = "5"
  alarm_description   = "B2A API Lambda has high error rate"
  alarm_actions       = [aws_sns_topic.b2a_alerts.arn]

  dimensions = {
    FunctionName = aws_lambda_function.b2a_api.function_name
  }
}

# Alarm for Slasher errors
resource "aws_cloudwatch_metric_alarm" "slasher_errors" {
  alarm_name          = "b2a-slasher-errors-${var.environment}"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "Errors"
  namespace           = "AWS/Lambda"
  period              = "300"
  statistic           = "Sum"
  threshold           = "3"
  alarm_description   = "B2A Slasher has repeated errors"
  alarm_actions       = [aws_sns_topic.b2a_alerts.arn]

  dimensions = {
    FunctionName = aws_lambda_function.b2a_slasher.function_name
  }
}

# Alarm for Sync errors
resource "aws_cloudwatch_metric_alarm" "sync_errors" {
  alarm_name          = "b2a-sync-errors-${var.environment}"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "Errors"
  namespace           = "AWS/Lambda"
  period              = "300"
  statistic           = "Sum"
  threshold           = "3"
  alarm_description   = "B2A Sync worker has repeated failures - deposits may not be credited"
  alarm_actions       = [aws_sns_topic.b2a_alerts.arn]

  dimensions = {
    FunctionName = aws_lambda_function.b2a_sync.function_name
  }
}

output "alerts_sns_topic_arn" {
  value = aws_sns_topic.b2a_alerts.arn
}