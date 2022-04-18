INSERT INTO "public"."biominer_indexd_file" ("guid", "filename", "size", "created_at", "updated_at", "status", "rev", "baseid", "uploader", "version") VALUES ('biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88', 'R20061200-L17-25-B140353-TT_combined_R2.fastq.gz', 102400, EXTRACT(EPOCH from now()) * 1000, EXTRACT(EPOCH from now()) * 1000, 'validated', 'no_rev', '4ec4d151-061b-4bcb-ad3a-425c712bfc88', 'biominer-admin', 1);

INSERT INTO "public"."biominer_indexd_alias" ("name", "file") VALUES ('doi:10.1109/5.771073', 'biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88');
INSERT INTO "public"."biominer_indexd_alias" ("name", "file") VALUES ('QUAR-A002-A-010-A001-0182-010-01', 'biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88');

INSERT INTO "public"."biominer_indexd_hash" ("hash", "hash_type", "file") VALUES ('e59ff97941044f85df5297e1c302d260', 'md5', 'biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88');
INSERT INTO "public"."biominer_indexd_hash" ("hash", "hash_type", "file") VALUES ('d2a84f4b8b650937ec8f73cd8be2c74add5a911ba64df27458ed8229da804a26', 'sha256', 'biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88');

INSERT INTO "public"."biominer_indexd_url" ("url", "created_at", "status", "uploader", "file") VALUES ('oss://pgx-source-data/CBCGA2020/RNA-seq/Project_s187g01098_123Samples_20201227/Sample_R20061199-L17-24-B140351-TT/R20061200-L17-25-B140353-TT_combined_R2.fastq.gz', EXTRACT(EPOCH from now()) * 1000, 'validated', 'biominer-admin', 'biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88');
INSERT INTO "public"."biominer_indexd_url" ("url", "created_at", "status", "uploader", "file") VALUES ('node://CBCGA/R20061200-L17-25-B140353-TT_combined_R2.fastq.gz', EXTRACT(EPOCH from now()) * 1000, 'validated', 'biominer-admin', 'biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88');
INSERT INTO "public"."biominer_indexd_url" ("url", "created_at", "status", "uploader", "file") VALUES ('gsa://CBCGA/R20061200-L17-25-B140353-TT_combined_R2.fastq.gz', EXTRACT(EPOCH from now()) * 1000, 'validated', 'biominer-admin', 'biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88');

INSERT INTO "public"."biominer_indexd_tag" ("field_name", "field_value", "file") VALUES ('project_name', 'CBCGA', 'biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88');
INSERT INTO "public"."biominer_indexd_tag" ("field_name", "field_value", "file") VALUES ('collection_date', '2021-07-10', 'biominer.fudan-pgx/3ec4d151-061b-4bcb-ad3a-425c712bfc88');

INSERT INTO "public"."biominer_indexd_file" ("guid", "filename", "size", "created_at", "updated_at", "status", "rev", "baseid", "uploader", "version") VALUES ('biominer.fudan-pgx/fg44d151-061b-4bcb-ad3a-425c712bfc88', 'R20061201-L17-25-B140353-TT_combined_R2.fastq.gz', 102400, EXTRACT(EPOCH from now()) * 1000, EXTRACT(EPOCH from now()) * 1000, 'validated', 'no_rev', '5ec4d151-061b-4bcb-ad3a-425c712bfc88', 'biominer-admin', 1);

INSERT INTO "public"."biominer_indexd_alias" ("name", "file") VALUES ('doi:10.1109/5.771074', 'biominer.fudan-pgx/fg44d151-061b-4bcb-ad3a-425c712bfc88');
INSERT INTO "public"."biominer_indexd_alias" ("name", "file") VALUES ('QUAR-A002-A-010-A001-0183-010-01', 'biominer.fudan-pgx/fg44d151-061b-4bcb-ad3a-425c712bfc88');

INSERT INTO "public"."biominer_indexd_hash" ("hash", "hash_type", "file") VALUES ('559ff97941044f85df5297e1c302d260', 'md5', 'biominer.fudan-pgx/fg44d151-061b-4bcb-ad3a-425c712bfc88');
INSERT INTO "public"."biominer_indexd_hash" ("hash", "hash_type", "file") VALUES ('62a84f4b8b650937ec8f73cd8be2c74add5a911ba64df27458ed8229da804a26', 'sha256', 'biominer.fudan-pgx/fg44d151-061b-4bcb-ad3a-425c712bfc88');

INSERT INTO "public"."biominer_indexd_url" ("url", "created_at", "status", "uploader", "file") VALUES ('oss://pgx-source-data/CBCGA2020/RNA-seq/Project_s187g01098_123Samples_20201227/Sample_R20061199-L17-24-B140351-TT/R20061201-L17-25-B140353-TT_combined_R2.fastq.gz', EXTRACT(EPOCH from now()) * 1000, 'validated', 'biominer-admin', 'biominer.fudan-pgx/fg44d151-061b-4bcb-ad3a-425c712bfc88');
INSERT INTO "public"."biominer_indexd_url" ("url", "created_at", "status", "uploader", "file") VALUES ('node://CBCGA/R20061201-L17-25-B140353-TT_combined_R2.fastq.gz', EXTRACT(EPOCH from now()) * 1000, 'validated', 'biominer-admin', 'biominer.fudan-pgx/fg44d151-061b-4bcb-ad3a-425c712bfc88');
INSERT INTO "public"."biominer_indexd_url" ("url", "created_at", "status", "uploader", "file") VALUES ('gsa://CBCGA/R20061201-L17-25-B140353-TT_combined_R2.fastq.gz', EXTRACT(EPOCH from now()) * 1000, 'validated', 'biominer-admin', 'biominer.fudan-pgx/fg44d151-061b-4bcb-ad3a-425c712bfc88');

INSERT INTO "public"."biominer_indexd_tag" ("field_name", "field_value", "file") VALUES ('project_name', 'CBCGA', 'biominer.fudan-pgx/fg44d151-061b-4bcb-ad3a-425c712bfc88');
INSERT INTO "public"."biominer_indexd_tag" ("field_name", "field_value", "file") VALUES ('collection_date', '2021-07-10', 'biominer.fudan-pgx/fg44d151-061b-4bcb-ad3a-425c712bfc88');
