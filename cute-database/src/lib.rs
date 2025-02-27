mod standalone;
mod generate_mysql;

pub struct DBConnectorConfig {
    pub id : String,
    pub password : String,
    pub ip_addr : String,
    pub port_number : u32,
    pub database : String,
}

pub enum DBConnector {
    DBLocal {
        db_config : DBConnectorConfig
    },
    DBMysql {
        db_config : DBConnectorConfig
    }
}

impl DBConnector {
    pub fn create_local(db_config : DBConnectorConfig) -> Self {
        Self::DBLocal {
            db_config,
        }
    }

    pub fn create_mysql(db_config : DBConnectorConfig) -> Self {
        Self::DBMysql {
            db_config,
        }
    }

    pub fn connect_database(&self) {

    }
}