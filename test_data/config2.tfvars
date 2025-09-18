shared_config = {
  cache = {
    type = "redis"
    ttl  = 3600
  }
  
  monitoring = {
    enabled = true
    interval = 60
  }
}

tags = {
  Environment = "staging"
}

network_settings = {
  vpc_cidr = "10.0.0.0/16"
}
