initSidebarItems({"fn":[["arrow_to_parquet_schema","Convert arrow schema to parquet schema"],["parquet_to_arrow_field","Convert parquet column schema to arrow field."],["parquet_to_arrow_schema","Convert Parquet schema to Arrow schema including optional metadata. Attempts to decode any existing Arrow schema metadata, falling back to converting the Parquet schema column-wise"],["parquet_to_arrow_schema_by_columns","Convert parquet schema to arrow schema including optional metadata, only preserving some leaf columns."],["parquet_to_arrow_schema_by_root_columns","Convert parquet schema to arrow schema including optional metadata, only preserving some root columns. This is useful if we have columns `a.b`, `a.c.e` and `a.d`, and want `a` with all its child fields"]]});