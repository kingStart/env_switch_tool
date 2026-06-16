import { createContext, useContext } from "react";

export type Locale = "en" | "zh";

export interface Messages {
  app_title: string;
  active_count: string;
  new_group: string;
  select_group: string;
  select_group_hint: string;
  no_groups: string;
  no_groups_hint: string;
  vars_count: string;
  hosts_count: string;
  enable: string;
  disable: string;
  delete_confirm: string;
  // Group editor
  active: string;
  inactive: string;
  key: string;
  value: string;
  mode: string;
  add: string;
  no_vars: string;
  no_vars_hint: string;
  no_hosts: string;
  no_hosts_hint: string;
  mode_override: string;
  mode_prepend: string;
  mode_append: string;
  remove: string;
  ip_address: string;
  hostname: string;
  // Create modal
  create_group: string;
  group_name: string;
  group_name_placeholder: string;
  description: string;
  description_placeholder: string;
  priority: string;
  priority_hint: string;
  group_kind: string;
  kind_env: string;
  kind_hosts: string;
  cancel: string;
  create: string;
  // Settings
  theme: string;
  theme_light: string;
  theme_dark: string;
  theme_system: string;
  language: string;
  // Profiles
  profiles: string;
  no_profiles: string;
  create_profile: string;
  profile_name: string;
  profile_groups: string;
  activate: string;
  deactivate: string;
  sync_hosts: string;
}

const en: Messages = {
  app_title: "EnvTools",
  active_count: "{active}/{total} active",
  new_group: "New Group",
  select_group: "Select a group to manage",
  select_group_hint: "Or create a new group to get started",
  no_groups: "No groups yet",
  no_groups_hint: "Click + to create one",
  vars_count: "{n} vars",
  hosts_count: "{n} hosts",
  enable: "Enable",
  disable: "Disable",
  delete_confirm: 'Delete "{name}"?',
  active: "Active",
  inactive: "Inactive",
  key: "Key",
  value: "Value",
  mode: "Mode",
  add: "Add",
  no_vars: "No variables yet",
  no_vars_hint: "Use the form above to add key-value pairs",
  no_hosts: "No hosts entries yet",
  no_hosts_hint: "Use the form above to add IP-hostname mappings",
  mode_override: "Override",
  mode_prepend: "Prepend",
  mode_append: "Append",
  remove: "Remove",
  ip_address: "IP Address",
  hostname: "Hostname",
  create_group: "Create Group",
  group_name: "Group Name",
  group_name_placeholder: "e.g. nodejs-dev, local-dns",
  description: "Description",
  description_placeholder: "What is this group for?",
  priority: "Priority",
  priority_hint: "higher wins conflicts",
  group_kind: "Type",
  kind_env: "Environment Variables",
  kind_hosts: "Hosts Mapping",
  cancel: "Cancel",
  create: "Create",
  theme: "Theme",
  theme_light: "Light",
  theme_dark: "Dark",
  theme_system: "System",
  language: "Language",
  profiles: "Profiles",
  no_profiles: "No profiles yet",
  create_profile: "Create Profile",
  profile_name: "Profile Name",
  profile_groups: "Groups",
  activate: "Activate",
  deactivate: "Deactivate",
  sync_hosts: "Sync Hosts",
};

const zh: Messages = {
  app_title: "EnvTools",
  active_count: "{active}/{total} 已启用",
  new_group: "新建分组",
  select_group: "选择分组以管理",
  select_group_hint: "或创建新分组开始使用",
  no_groups: "暂无分组",
  no_groups_hint: "点击 + 创建一个",
  vars_count: "{n} 个变量",
  hosts_count: "{n} 条映射",
  enable: "启用",
  disable: "禁用",
  delete_confirm: '确定删除 "{name}" 吗？',
  active: "已启用",
  inactive: "未启用",
  key: "键名",
  value: "值",
  mode: "模式",
  add: "添加",
  no_vars: "暂无环境变量",
  no_vars_hint: "使用上方表单添加键值对",
  no_hosts: "暂无域名映射",
  no_hosts_hint: "使用上方表单添加 IP 和域名映射",
  mode_override: "覆盖",
  mode_prepend: "前置",
  mode_append: "追加",
  remove: "删除",
  ip_address: "IP 地址",
  hostname: "域名",
  create_group: "创建分组",
  group_name: "分组名称",
  group_name_placeholder: "例如 nodejs-dev, local-dns",
  description: "描述",
  description_placeholder: "这个分组用来做什么？",
  priority: "优先级",
  priority_hint: "高优先级覆盖低优先级",
  group_kind: "类型",
  kind_env: "环境变量",
  kind_hosts: "域名映射",
  cancel: "取消",
  create: "创建",
  theme: "主题",
  theme_light: "浅色",
  theme_dark: "深色",
  theme_system: "跟随系统",
  language: "语言",
  profiles: "场景",
  no_profiles: "暂无场景",
  create_profile: "创建场景",
  profile_name: "场景名称",
  profile_groups: "关联分组",
  activate: "启用",
  deactivate: "停用",
  sync_hosts: "同步 Hosts",
};

const locales: Record<Locale, Messages> = { en, zh };

export function getMessages(locale: Locale): Messages {
  return locales[locale];
}

export function t(msg: string, params?: Record<string, string | number>): string {
  if (!params) return msg;
  return Object.entries(params).reduce(
    (s, [k, v]) => s.replace(`{${k}}`, String(v)),
    msg
  );
}

export interface I18nContext {
  locale: Locale;
  messages: Messages;
  setLocale: (l: Locale) => void;
}

export const I18nCtx = createContext<I18nContext>({
  locale: "en",
  messages: en,
  setLocale: () => {},
});

export function useI18n() {
  return useContext(I18nCtx);
}
