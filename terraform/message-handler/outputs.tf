output "consume_endpoint_url" { value = "${aws_lambda_function_url.this.function_url}message/consume" }
output "create_endpoint_url" { value = "${aws_lambda_function_url.this.function_url}message/create" }
