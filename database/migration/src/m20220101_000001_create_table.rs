use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InspirationImage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InspirationImage::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InspirationImage::SourceUrl)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InspirationImage::SourceId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InspirationImage::Description).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InspirationImage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum InspirationImage {
    Table,
    Id,
    SourceId,
    SourceUrl,
    Description,
}
