CREATE TABLE IF NOT EXISTS "test_assets" (
  "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  "title" VARCHAR(200) NOT NULL,
  "attachment" TEXT NOT NULL,
  "attachment_filename" TEXT,
  "attachment_mime_type" TEXT,
  "attachment_size" BIGINT,
  "created_at" TIMESTAMPTZ DEFAULT NOW(),
  "updated_at" TIMESTAMPTZ DEFAULT NOW()
);
