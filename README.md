# SSR-backend

### 流程或系統品質提升措施

- 使用 JWT 驗證使用者身分和一些必要資訊
- 驗證資料，避免錯誤資訊
- 遵守 commit message 規範
- 完整記錄 log 並且輸出到終端
    > <img src="./log.png" height="360px" width="480px" />
- 使用 sqlx 的 pooling 代替每次查詢都對資料庫連接
- 使用 docker 包裝
