use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let result = manager
            .create_table(
                Table::create()
                    .table(ClockInfos::Table)
                    .col(
                        ColumnDef::new(ClockInfos::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ClockInfos::Clock).string().not_null())
                    .col(ColumnDef::new(ClockInfos::ClockHash).char_len(64).unique_key().not_null())
                    .col(ColumnDef::new(ClockInfos::NodeId).char_len(64).not_null())
                    .col(ColumnDef::new(ClockInfos::MessageId).char_len(64).not_null())
                    .col(ColumnDef::new(ClockInfos::EventCount).integer().not_null())
                    .col(ColumnDef::new(ClockInfos::CreateAt).timestamp())
                    .to_owned(),
            ).await;

        if let Err(err) = result {
            return Err(err);
        }

        // create index
        let msgid_index = Index::create()
            .if_not_exists()
            .name("idx-clockinfos-messageid")
            .table(ClockInfos::Table)
            .col(ClockInfos::MessageId)
            .to_owned();
        let result = manager.create_index(msgid_index).await;
        if let Err(err) = result {
            return Err(err);
        }

        let nodeid_index = Index::create()
            .if_not_exists()
            .name("idx-clockinfos-nodeid")
            .table(ClockInfos::Table)
            .col(ClockInfos::NodeId)
            .to_owned();
        manager.create_index(nodeid_index).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ClockInfos::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ClockInfos {
    Table,
    Id,
    Clock,
    ClockHash,
    NodeId,
    MessageId,
    EventCount,
    CreateAt,
}
