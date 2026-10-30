# 静态书库

网页书架读取 public/library/catalog.json。每本书的版本目录保存 manifest.json、正文包、分析计划、查询索引、分析分片、词典与语法内容包，以及封面和插图。

每个分析分片包含六个桌面分析单元。打开图书、滚动和章节跳转按范围读取；会话保留最近四片。词典正文独立去重，各表记引用对应的词条。查询与内容包按约 1 MiB 原始 JSON 分包，使用 gzip 分发。

阅读进度、摘录、笔记、词汇状态、颜色及标签保存在浏览器。导入与即时分析入口在静态页面中禁用。语法指南通过书架右上角按钮进入。

## 生成与构建

先执行 cargo build --release -p readux-core --bin readux-nlp，再执行 npm run generate:library 和 npm run build。

生成器默认读取 Windows 文档目录中的 ReadUX Library。参数 --title 指定书名，--library 指定书库，--service 指定分析服务，--version 指定资源版本。文本或规则更新后采用新版本目录。

生成过程保存在 .agents/analysis/static-library 中，相同参数支持续接；report.json 保存最终数量与体积。公开书目在全部资源完成后更新。

## 预览与上传

开发预览使用 npm run static:dev，生产预览使用 npm run preview。EdgeOne Pages 的构建命令保持 npm run build，输出目录保持 dist。直接上传整个 dist 文件夹；ZIP 的根目录应包含 index.html，library 目录随页面一起上传。

浏览器根据 gzip 文件头解压资源，同时支持平台已经解压的 JSON 响应。

完整资源核验使用 node scripts/verify_static_library.mjs，覆盖全部查询引用、全文分片、正式胶囊投影和平台文件限制，报告保存在 .agents/analysis/static-library/verification.json。
