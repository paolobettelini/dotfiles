local on_attach = require("plugins.configs.lspconfig").on_attach
local capabilities = require("plugins.configs.lspconfig").capabilities
local lspconfig = require("lspconfig")
local util = require "lspconfig/util"

lspconfig.texlab.setup({
    on_attach = on_attach,
    capabilities = capabilities,
    filetypes = {"tex"},
})