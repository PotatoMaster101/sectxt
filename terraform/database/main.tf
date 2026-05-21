resource "neon_project" "this" {
  history_retention_seconds = 21600
  name                      = "${var.project}_${var.environment}"
  region_id                 = var.neon_region
}

data "neon_project" "this" {
  id = neon_project.this.id
}

resource "neon_role" "this" {
  branch_id  = neon_project.this.default_branch_id
  name       = "admin"
  project_id = neon_project.this.id
}

resource "neon_database" "postgres_db" {
  branch_id  = neon_project.this.default_branch_id
  name       = var.project
  owner_name = neon_role.this.name
  project_id = neon_project.this.id
}
