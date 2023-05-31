use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .create_table(
                Table::create()
                    .table(Index::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Index::Name)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Index::EmbeddingModel).string().not_null())
                    .col(ColumnDef::new(Index::TextSplitter).string().not_null())
                    .col(ColumnDef::new(Index::VectorDb).string().not_null())
                    .col(ColumnDef::new(Index::VectorDbParams).json())
                    .col(ColumnDef::new(Index::UniqueParams).json())
                    .to_owned(),
            )
            .await?;

        let _ = manager
            .create_table(
                Table::create()
                    .table(Content::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Content::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Content::IndexName).string().not_null())
                    .col(ColumnDef::new(Content::Text).text().not_null())
                    .col(ColumnDef::new(Content::ContentType).string().not_null())
                    .col(ColumnDef::new(Content::Metadata).json())
                    .col(ColumnDef::new(Content::EmbeddingStatus).string())
                    .to_owned(),
            )
            .await;

        manager
            .create_table(
                Table::create()
                    .table(IndexChunks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IndexChunks::ChunkId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(IndexChunks::ContentId).string().not_null())
                    .col(ColumnDef::new(IndexChunks::Text).text().not_null())
                    .col(ColumnDef::new(IndexChunks::IndexName).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Index::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Index {
    Table,
    Name,
    TextSplitter,
    EmbeddingModel,
    VectorDb,
    VectorDbParams,
    UniqueParams,
}

#[derive(Iden)]
enum IndexChunks {
    Table,
    ContentId,
    ChunkId,
    Text,
    IndexName,
}

#[derive(Iden)]
enum Content {
    Table,
    Id,
    IndexName,
    ContentType,
    Text,
    Metadata,
    EmbeddingStatus,
}
