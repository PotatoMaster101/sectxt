locals {
  function_name = "${var.project}-${var.environment}-handle-message"
  policies = {
    lambda = aws_iam_policy.this.arn
    logs   = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
  }
}

resource "aws_lambda_function" "this" {
  architectures                  = ["arm64"]
  filename                       = var.sectxt_message_path
  function_name                  = local.function_name
  handler                        = "bootstrap"
  memory_size                    = 128
  role                           = aws_iam_role.this.arn
  runtime                        = "provided.al2023"
  source_code_hash               = filebase64sha256(var.sectxt_message_path)
  timeout                        = 15

  environment {
    variables = {
      DATABASE_URL = var.database_url
    }
  }
}

resource "aws_iam_policy" "this" {
  name = local.function_name
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents",
        ],
        Effect   = "Allow",
        Resource = "arn:aws:logs:*:*:*"
      },
    ]
  })
}

resource "aws_iam_role" "this" {
  name = local.function_name
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action    = "sts:AssumeRole"
        Effect    = "Allow"
        Principal = { Service = "lambda.amazonaws.com" }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "this" {
  for_each   = local.policies
  policy_arn = each.value
  role       = aws_iam_role.this.name
}

resource "aws_iam_role_policy_attachments_exclusive" "this" {
  policy_arns = values(local.policies)
  role_name   = aws_iam_role.this.name
}

resource "aws_cloudwatch_log_group" "this" {
  name              = "/aws/lambda/${local.function_name}"
  retention_in_days = 14
}

resource "aws_lambda_function_url" "this" {
  authorization_type = "NONE"
  function_name      = aws_lambda_function.this.function_name
}
