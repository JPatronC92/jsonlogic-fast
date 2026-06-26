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
  description = "RPC URL for the blockchain network. For prod/mainnet use https://mainnet.base.org (or your provider). Set via TF_VAR or secrets for prod."
  type        = string
  default     = "https://sepolia.base.org"
}

variable "contract_address" {
  description = "Address of the B2AStaking contract. For Base Mainnet prod set to the real deployed address (see go-live checklist)."
  type        = string
  default     = "0x0000000000000000000000000000000000000000" # Replace in production
}

variable "private_key" {
  description = "DEPRECATED - Private key now loaded from AWS Secrets Manager (see b2a/slasher-private-key-<env>) in the Terraform manifests. Kept for backward compatibility only."
  type        = string
  sensitive   = true
  default     = ""  # No longer used
}

variable "sync_schedule" {
  description = "EventBridge schedule expression for the deposit sync worker (e.g. rate(2 minutes))"
  type        = string
  default     = "rate(2 minutes)"
}

variable "siwe_domain" {
  description = "Expected SIWE domain for B2A API auth"
  type        = string
  default     = ""
}

variable "siwe_uri" {
  description = "Expected SIWE URI for B2A API auth"
  type        = string
  default     = ""
}

variable "siwe_chain_id" {
  description = "Expected SIWE chain id"
  type        = string
  default     = "1"
}

variable "siwe_max_age_secs" {
  description = "Maximum accepted SIWE message age in seconds"
  type        = string
  default     = "3600"
}
