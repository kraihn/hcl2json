shared_config = {
  database = {
    engine = "mysql"
    port   = 3306
  }
  
  logging = {
    level = "info"
    format = "json"
  }
}

tags = {
  Team = "backend"
}

app_settings = {
  timeout = 30
}
