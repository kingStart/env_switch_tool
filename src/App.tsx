import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { GroupList } from "./components/GroupList";
import { GroupEditor } from "./components/GroupEditor";
import { CreateGroupModal } from "./components/CreateGroupModal";
import { SettingsPanel } from "./components/SettingsPanel";
import { useTheme } from "./hooks/useTheme";
import { I18nCtx, getMessages, t, type Locale } from "./i18n";

interface GroupInfo {
  name: string;
  description: string;
  active: boolean;
  priority: number;
  variable_count: number;
}

export default function App() {
  const { theme, setTheme } = useTheme();
  const [locale, setLocaleState] = useState<Locale>(
    () => (localStorage.getItem("envtools-locale") as Locale) || "en"
  );
  const messages = useMemo(() => getMessages(locale), [locale]);
  const setLocale = useCallback((l: Locale) => {
    setLocaleState(l);
    localStorage.setItem("envtools-locale", l);
  }, []);

  const [groups, setGroups] = useState<GroupInfo[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadGroups = useCallback(async () => {
    try {
      const result = await invoke<GroupInfo[]>("get_groups");
      setGroups(result);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadGroups();
  }, [loadGroups]);

  const handleToggle = useCallback(async (name: string, active: boolean) => {
    try {
      await invoke(active ? "disable_group" : "enable_group", { name });
      await loadGroups();
    } catch (e) {
      setError(String(e));
    }
  }, [loadGroups]);

  const handleDelete = useCallback(async (name: string) => {
    try {
      await invoke("delete_group", { name });
      setSelectedGroup((prev) => (prev === name ? null : prev));
      await loadGroups();
    } catch (e) {
      setError(String(e));
    }
  }, [loadGroups]);

  const handleCreate = useCallback(async (name: string, description: string, priority: number) => {
    try {
      await invoke("create_group", { name, description, priority });
      setShowCreateModal(false);
      await loadGroups();
      setSelectedGroup(name);
    } catch (e) {
      setError(String(e));
    }
  }, [loadGroups]);

  const activeCount = useMemo(() => groups.filter((g) => g.active).length, [groups]);

  const i18nValue = useMemo(
    () => ({ locale, messages, setLocale }),
    [locale, messages, setLocale]
  );

  return (
    <I18nCtx.Provider value={i18nValue}>
      <div className="h-screen flex flex-col bg-slate-50 dark:bg-gray-900 transition-colors duration-200">
        {/* Error Toast */}
        {error && (
          <div className="fixed top-4 right-4 z-50 max-w-sm bg-red-500 text-white px-4 py-3 rounded-xl shadow-lg flex items-center gap-3">
            <span className="text-sm flex-1">{error}</span>
            <button onClick={() => setError(null)} className="text-white/80 hover:text-white shrink-0">
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        )}

        <div className="flex flex-1 overflow-hidden">
          {/* Left Panel */}
          <aside className="w-72 bg-white dark:bg-gray-800 border-r border-slate-200 dark:border-gray-700 flex flex-col transition-colors">
            <div className="px-5 py-4 border-b border-slate-100 dark:border-gray-700">
              <div className="flex items-center justify-between">
                <div>
                  <h1 className="text-base font-bold text-slate-800 dark:text-white tracking-tight">
                    {messages.app_title}
                  </h1>
                  <p className="text-xs text-slate-400 dark:text-gray-500 mt-0.5">
                    {t(messages.active_count, { active: activeCount, total: groups.length })}
                  </p>
                </div>
                <div className="flex items-center gap-1">
                  <button
                    onClick={() => setShowSettings(!showSettings)}
                    className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-slate-100 dark:hover:bg-gray-700 text-slate-400 transition-colors"
                    title="Settings"
                  >
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                  </button>
                  <button
                    onClick={() => setShowCreateModal(true)}
                    className="w-8 h-8 flex items-center justify-center rounded-lg bg-indigo-500 hover:bg-indigo-600 text-white transition-colors shadow-sm"
                    title={messages.new_group}
                  >
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M12 4v16m8-8H4" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>

            <div className="flex-1 overflow-y-auto scrollbar-thin">
              {showSettings ? (
                <SettingsPanel theme={theme} setTheme={setTheme} />
              ) : (
                <GroupList
                  groups={groups}
                  selectedGroup={selectedGroup}
                  onSelect={setSelectedGroup}
                  onToggle={handleToggle}
                  onDelete={handleDelete}
                />
              )}
            </div>
          </aside>

          {/* Right Panel */}
          <main className="flex-1 overflow-y-auto scrollbar-thin bg-slate-50 dark:bg-gray-900 transition-colors">
            {selectedGroup ? (
              <GroupEditor groupName={selectedGroup} onUpdate={loadGroups} />
            ) : (
              <div className="flex flex-col items-center justify-center h-full text-slate-400 dark:text-gray-500">
                <svg className="w-16 h-16 mb-4 text-slate-200 dark:text-gray-700" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                </svg>
                <p className="text-sm font-medium">{messages.select_group}</p>
                <p className="text-xs mt-1 text-slate-300 dark:text-gray-600">{messages.select_group_hint}</p>
              </div>
            )}
          </main>
        </div>

        {showCreateModal && (
          <CreateGroupModal
            onClose={() => setShowCreateModal(false)}
            onCreate={handleCreate}
          />
        )}
      </div>
    </I18nCtx.Provider>
  );
}
