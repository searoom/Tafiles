export const imageExts = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg']

export const officeExts = {
  pdf: 'pdf',
  xlsx: 'excel',
  xls: 'excel',
  docx: 'docx',
  doc: 'doc',
  ppt: 'ppt',
  pptx: 'pptx',
}

export const plainExts = ['txt', 'log', 'ini', 'cfg', 'env', 'conf', 'yaml', 'yml', 'toml']

export const codeExts = {
  js: 'javascript',
  ts: 'typescript',
  jsx: 'jsx',
  tsx: 'tsx',
  py: 'python',
  html: 'html',
  htm: 'html',
  css: 'css',
  scss: 'css',
  less: 'css',
  json: 'json',
  xml: 'xml',
  xhtml: 'xml',
  md: 'markdown',
  sql: 'sql',
  java: 'java',
  cpp: 'cpp',
  c: 'c',
  cs: 'csharp',
  php: 'php',
  rb: 'ruby',
  go: 'go',
  rs: 'rust',
  swift: 'swift',
  kt: 'kotlin',
  sh: 'shell',
  bash: 'shell',
  bat: 'bat',
  ps1: 'powershell',
  zsh: 'shell',
  pl: 'perl',
  lua: 'lua',
  r: 'r',
  dart: 'dart',
  vue: 'vue',
  svelte: 'svelte',
}

export function getFileType(ext) {
  const e = (ext || '').toLowerCase().replace(/^\./, '')
  if (!e) return 'unknown'
  if (imageExts.includes(e)) return 'image'
  if (e === 'pdf') return 'pdf'
  if (e === 'xlsx' || e === 'xls') return 'excel'
  if (e === 'docx' || e === 'doc') return 'docx'
  if (e === 'ppt' || e === 'pptx') return 'ppt'
  if (plainExts.includes(e)) return 'text'
  if (e in codeExts) return 'code'
  return 'unknown'
}

export function isImageExt(ext) {
  return imageExts.includes((ext || '').toLowerCase().replace(/^\./, ''))
}

export const fileTypeIcons = {
  image: 'image',
  pdf: 'pdf',
  excel: 'excel',
  docx: 'word',
  doc: 'word',
  ppt: 'powerpoint',
  code: 'code',
  text: 'text',
  unknown: 'file',
}
