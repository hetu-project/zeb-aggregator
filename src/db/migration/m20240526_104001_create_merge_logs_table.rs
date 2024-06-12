use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MergeLogs::Table)
                    .col(
                        ColumnDef::new(MergeLogs::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MergeLogs::FromId).char_len(64).not_null())
                    .col(ColumnDef::new(MergeLogs::ToId).char_len(64).not_null())
                    .col(ColumnDef::new(MergeLogs::StartCount).integer().not_null())
                    .col(ColumnDef::new(MergeLogs::EndCount).integer().not_null())
                    .col(ColumnDef::new(MergeLogs::SClockHash).char_len(64).not_null())
                    .col(ColumnDef::new(MergeLogs::EClockHash).char_len(64).not_null())
                    .col(ColumnDef::new(MergeLogs::MergeAt).timestamp().not_null())
                    .col(ColumnDef::new(MergeLogs::NodeId).char_len(64).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MergeLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum MergeLogs {
    Table,
    Id,
    FromId,
    ToId,
    StartCount,
    EndCount,
    SClockHash,
    EClockHash,
    MergeAt,
    NodeId
}