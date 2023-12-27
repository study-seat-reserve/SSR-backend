# 使用 Rust 官方 Docker 映像作為基礎映像
FROM rust:1.67 as builder

# 安裝 sqlite3
RUN apt-get update && apt-get install -y sqlite3

# 創建並設定工作目錄
WORKDIR /usr/src/study_seat_reserve

# 複製 Cargo.toml 和 Cargo.lock 檔案
COPY Cargo.toml Cargo.lock ./

# 創建一個假的 main.rs 來獲取依賴項
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 構建依賴項
RUN cargo build --release

# 複製程式碼和 SQL 檔案
COPY src ./src
COPY create_tables.sql ./

# 使用 sqlite3 執行 SQL 檔案，創建資料庫
RUN sqlite3 SSR.db3 < create_tables.sql

# DB路徑，用於 cargo build 時 sqlx 可以找的到資料庫
ENV DATABASE_URL=sqlite:./SSR.db3

# 構建應用程式
RUN touch src/main.rs && cargo build --release

# 第二階段：創建運行映像
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y openssl libssl1.1 && rm -rf /var/lib/apt/lists/*

# 複製執行檔和資料庫檔案
COPY --from=builder /usr/src/study_seat_reserve/target/release/study_seat_reserve /usr/src/study_seat_reserve/study_seat_reserve
COPY --from=builder /usr/src/study_seat_reserve/SSR.db3 /usr/src/study_seat_reserve/SSR.db3
COPY Rocket.toml /usr/src/study_seat_reserve/Rocket.toml

WORKDIR /usr/src/study_seat_reserve

# 設置環境變數
COPY .env.release /usr/src/study_seat_reserve/.env
ENV DATABASE_URL=sqlite:./SSR.db3


# 設定容器啟動時運行的命令
CMD ["/usr/src/study_seat_reserve/study_seat_reserve"]