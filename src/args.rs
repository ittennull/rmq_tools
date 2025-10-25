use clap::Parser;

#[derive(Parser)]
#[command()]
pub struct Args {
    #[arg(
        short,
        long,
        default_value = "http://guest:guest@localhost:15672/api",
        help = "URL to RabbitMQ API"
    )]
    pub url: String,

    #[arg(short, long, default_value = "/", help = "Virtual host")]
    pub vhost: String,
    
    #[arg(short, long, default_value_t = 3000, help = "Port number to run the UI on")]
    pub port: u16
}