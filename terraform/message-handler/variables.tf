variable "aws_region" { type = string }
variable "database_url" {
  sensitive = true
  type      = string
}
variable "environment" { type = string }
variable "project" { type = string }
variable "sectxt_message_path" { type = string }
