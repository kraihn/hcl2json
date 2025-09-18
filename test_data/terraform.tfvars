region = "us-west-2"
instance_type = "t3.micro"
enable_monitoring = true
port_numbers = [80, 443, 8080]

tags = {
  Environment = "production"
  Project     = "web-app"
  Owner       = "devops-team"
}

database = {
  engine = "mysql"
  version = "8.0"
  storage = 100
}
