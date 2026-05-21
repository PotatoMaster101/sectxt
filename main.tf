variable "aws_region" { default = "ap-southeast-2" }
variable "environment" { default = "dev" }
variable "neon_api_key" {
  type      = string
  sensitive = true
}
variable "project" { default = "sectxt" }

module "database" {
  environment  = var.environment
  neon_api_key = var.neon_api_key
  neon_region  = "aws-${var.aws_region}"
  project      = var.project
  source       = "./terraform/database"
}

module "message_handler" {
  aws_region          = var.aws_region
  database_url        = module.database.database_url
  environment         = var.environment
  project             = var.project
  sectxt_message_path = "${path.root}/target/lambda/sectxt-lambda-message/bootstrap.zip"
  source              = "./terraform/message-handler"
}

output "create_endpoint_url" { value = module.message_handler.create_endpoint_url }
output "consume_endpoint_url" { value = module.message_handler.consume_endpoint_url }
output "database_url" {
  sensitive = true
  value     = module.database.database_url
}
