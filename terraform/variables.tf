variable "aws_region" {
  description = "AWS Region to deploy to"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Deployment environment (e.g., dev, prod)"
  type        = string
  default     = "dev"
}

variable "balances_table_name" {
  description = "Name of the DynamoDB balances table"
  type        = string
  default     = "B2A_Balances"
}

variable "nonces_table_name" {
  description = "Name of the DynamoDB nonces table"
  type        = string
  default     = "B2A_Nonces"
}

variable "rpc_url" {
  description = "RPC URL for the blockchain network"
  type        = string
  default     = "https://sepolia.base.org"
}

variable "contract_address" {
  description = "Address of the B2AStaking contract"
  type        = string
  default     = "0x0000000000000000000000000000000000000000" # Replace in production
}

variable "private_key" {
  description = "Private key for slasher (use AWS Secrets Manager in prod)"
  type        = string
  sensitive   = true
  default     = "0000000000000000000000000000000000000000000000000000000000000001"
}
