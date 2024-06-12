use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let result = manager
            .create_table(
                Table::create()
                    .table(ZMessages::Table)
                    .col(
                        ColumnDef::new(ZMessages::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ZMessages::MessageId).char_len(64).not_null())
                    .col(ColumnDef::new(ZMessages::Version).integer())
                    .col(ColumnDef::new(ZMessages::Type).integer().not_null())
                    .col(ColumnDef::new(ZMessages::PublicKey).char_len(64))
                    .col(ColumnDef::new(ZMessages::Data).binary().not_null())
                    .col(ColumnDef::new(ZMessages::Signature).binary())
                    .col(ColumnDef::new(ZMessages::From).char_len(64).not_null())
                    .col(ColumnDef::new(ZMessages::To).char_len(64).not_null())
                    .col(ColumnDef::new(ZMessages::NodeId).char_len(64).not_null())
                    .to_owned(),
            )
            .await;

        if let Err(err) = result {
            return Err(err);
        }

        // create index
        let msgid_index = Index::create()
            .if_not_exists()
            .name("idx-zmessages-messageid")
            .table(ZMessages::Table)
            .col(ZMessages::MessageId)
            .to_owned();
        manager.create_index(msgid_index).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ZMessages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ZMessages {
    Table,
    Id,
    MessageId,
    Version,
    Type,
    PublicKey,
    Data,
    Signature,
    From,
    To,
    NodeId
}
