pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20240301_053139_create_generated_image;
mod m20240313_032654_add_descripiton_to_generated_image;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240301_053139_create_generated_image::Migration),
            Box::new(m20240313_032654_add_descripiton_to_generated_image::Migration),
        ]
    }
}
