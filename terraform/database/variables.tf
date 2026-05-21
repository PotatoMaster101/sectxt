variable "environment" { type = string }
variable "neon_api_key" {
  type      = string
  sensitive = true
}
variable "neon_region" { type = string }
variable "project" { type = string }
