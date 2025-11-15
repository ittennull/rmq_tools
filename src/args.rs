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
    pub port: u16,

    #[arg(long, help = "RabbitMQ server name to display in UI. By default it shows the host taken from url argument")]
    pub server_name: Option<String>,

    #[arg(long, default_value_t = 0, help = "Adds to UI a visual indication of how important the data on the connected server is. Values from 0 to 2")]
    pub importance_level: u8,
}