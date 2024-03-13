use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                sea_query::Table::alter()
                    .table(GeneratedImage::Table)
                    .add_column(
                        ColumnDef::new(GeneratedImage::Prompt)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .add_column(
                        ColumnDef::new(GeneratedImage::RevisedPrompt)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                sea_query::Table::alter()
                    .table(GeneratedImage::Table)
                    .drop_column(GeneratedImage::Prompt)
                    .drop_column(GeneratedImage::RevisedPrompt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum GeneratedImage {
    Table,
    RevisedPrompt,
    Prompt,
}
