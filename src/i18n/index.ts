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
  mode_override: string;
  mode_prepend: string;
  mode_append: string;
  remove: string;
  // Create modal
  create_group: string;
  group_name: string;
  group_name_placeholder: string;
  description: string;
  description_placeholder: string;
  priority: string;
  priority_hint: string;
  cancel: string;
  create: string;
  // Settings
  theme: string;
  theme_light: string;
  theme_dark: string;
  theme_system: string;
  language: string;
}

const en: Messages = {
  app_title: "EnvTools",
  active_count: "{active}/{total} active",
  new_group: "New Group",
  select_group: "Select a group to manage variables",
  select_group_hint: "Or create a new group to get started",
  no_groups: "No groups yet",
  no_groups_hint: "Click + to create one",
  vars_count: "{n} vars",
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
  mode_override: "Override",
  mode_prepend: "Prepend",
  mode_append: "Append",
  remove: "Remove",
  create_group: "Create Group",
  group_name: "Group Name",
  group_name_placeholder: "e.g. nodejs-dev, python-ml",
  description: "Description",
  description_placeholder: "What is this group for?",
  priority: "Priority",
  priority_hint: "higher wins conflicts",
  cancel: "Cancel",
  create: "Create",
  theme: "Theme",
  theme_light: "Light",
  theme_dark: "Dark",
  theme_system: "System",
  language: "Language",
};

const zh: Messages = {
  app_title: "EnvTools",
  active_count: "{active}/{total} 已启用",
  new_group: "新建分组",
  select_group: "选择分组以管理环境变量",
  select_group_hint: "或创建新分组开始使用",
  no_groups: "暂无分组",
  no_groups_hint: "点击 + 创建一个",
  vars_count: "{n} 个变量",
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
  mode_override: "覆盖",
  mode_prepend: "前置",
  mode_append: "追加",
  remove: "删除",
  create_group: "创建分组",
  group_name: "分组名称",
  group_name_placeholder: "例如 nodejs-dev, python-ml",
  description: "描述",
  description_placeholder: "这个分组用来做什么？",
  priority: "优先级",
  priority_hint: "高优先级覆盖低优先级",
  cancel: "取消",
  create: "创建",
  theme: "主题",
  theme_light: "浅色",
  theme_dark: "深色",
  theme_system: "跟随系统",
  language: "语言",
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
