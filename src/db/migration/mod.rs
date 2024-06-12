use sea_orm_migration::prelude::*;

mod m20240526_103941_create_clock_infos_table;
mod m20240526_104001_create_merge_logs_table;
mod m20240526_104014_create_node_info_table;
mod m20240526_104032_create_z_messages_table;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240526_103941_create_clock_infos_table::Migration),
            Box::new(m20240526_104001_create_merge_logs_table::Migration),
            Box::new(m20240526_104014_create_node_info_table::Migration),
            Box::new(m20240526_104032_create_z_messages_table::Migration),
        ]
    }
}