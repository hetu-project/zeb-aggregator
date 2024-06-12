use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NodeInfo::Table)
                    .col(
                        ColumnDef::new(NodeInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NodeInfo::NodeId).char_len(64).not_null())
                    .col(ColumnDef::new(NodeInfo::NeighborNodes).string().not_null())
                    .col(
                        ColumnDef::new(NodeInfo::IsAlive)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(NodeInfo::RPCDomain).char_len(64).not_null())
                    .col(
                        ColumnDef::new(NodeInfo::RPCPort)
                            .integer()
                            .not_null()
                            .default(0)
                    )
                    .col(ColumnDef::new(NodeInfo::WSDomain).char_len(64).not_null())
                    .col(
                        ColumnDef::new(NodeInfo::WSPort)
                            .integer()
                            .not_null()
                            .default(0)
                    )
                    .col(
                        ColumnDef::new(NodeInfo::ClockInfoIndex)
                            .integer()
                            .not_null()
                            .default(0)
                    )
                    .col(
                        ColumnDef::new(NodeInfo::MergeLogIndex)
                            .integer()
                            .not_null()
                            .default(0)
                    )
                    .col(
                        ColumnDef::new(NodeInfo::ZMessageIndex)
                            .integer()
                            .not_null()
                            .default(0)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NodeInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NodeInfo {
    Table,
    Id,
    NodeId,
    NeighborNodes,
    IsAlive,
    RPCDomain,
    RPCPort,
    WSDomain,
    WSPort,
    ClockInfoIndex,
    MergeLogIndex,
    ZMessageIndex,
}
