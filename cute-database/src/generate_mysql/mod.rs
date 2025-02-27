use cute_core::CuteError;
use crate::DBConnectorConfig;

pub struct MySQLComponent {

}

impl MySQLComponent {
    pub fn new(config : DBConnectorConfig) -> Result<MySQLComponent, CuteError> {
        let url = format!("mysql://{}/{}@{}:{}/{}",config.id,config.password,config.ip_addr,config.port_number,config.database);
        let pool  = mysql::Pool::new(url.as_str()).map_err(|e| CuteError::internal(e.to_string()))?;

        let connector = pool.get_conn().map_err(|e| CuteError::internal(e.to_string()))?;



        Ok(Self {

        })
    }
}

impl Drop for MySQLComponent {
    fn drop(&mut self) {
        todo!()
    }
}