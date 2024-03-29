use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GeneratedImage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GeneratedImage::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GeneratedImage::SourceUrl)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GeneratedImage::InspirationImageId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GeneratedImage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum GeneratedImage {
    Table,
    Id,
    SourceUrl,
    InspirationImageId,
}
