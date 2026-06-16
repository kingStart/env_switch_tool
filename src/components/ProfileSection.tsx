import { useState, useEffect, useCallback, memo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "../i18n";

interface ProfileInfo {
  name: string;
  description: string;
  group_names: string[];
}

interface Props {
  onProfileAction: () => void;
}

export const ProfileSection = memo(function ProfileSection({ onProfileAction }: Props) {
  const { messages } = useI18n();
  const [profiles, setProfiles] = useState<ProfileInfo[]>([]);
  const [expanded, setExpanded] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState("");
  const [newDesc, setNewDesc] = useState("");
  const [newGroups, setNewGroups] = useState("");

  const loadProfiles = useCallback(async () => {
    try {
      const result = await invoke<ProfileInfo[]>("get_profiles");
      setProfiles(result);
    } catch {
      // silent
    }
  }, []);

  useEffect(() => {
    loadProfiles();
  }, [loadProfiles]);

  const handleCreate = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim()) return;
    try {
      const groupNames = newGroups.split(",").map(s => s.trim()).filter(Boolean);
      await invoke("create_profile", { name: newName.trim(), description: newDesc, groupNames });
      setNewName("");
      setNewDesc("");
      setNewGroups("");
      setShowCreate(false);
      await loadProfiles();
    } catch {
      // silent
    }
  }, [newName, newDesc, newGroups, loadProfiles]);

  const handleActivate = useCallback(async (name: string) => {
    try {
      await invoke("activate_profile", { name });
      onProfileAction();
    } catch {
      // silent
    }
  }, [onProfileAction]);

  const handleDeactivate = useCallback(async (name: string) => {
    try {
      await invoke("deactivate_profile", { name });
      onProfileAction();
    } catch {
      // silent
    }
  }, [onProfileAction]);

  const handleDelete = useCallback(async (name: string) => {
    try {
      await invoke("delete_profile", { name });
      await loadProfiles();
    } catch {
      // silent
    }
  }, [loadProfiles]);

  return (
    <div className="border-b border-slate-100 dark:border-gray-700">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full px-5 py-3 flex items-center justify-between text-xs font-semibold text-slate-500 dark:text-gray-400 uppercase tracking-wider hover:bg-slate-50 dark:hover:bg-gray-700/30 transition-colors"
      >
        <span>{messages.profiles}</span>
        <svg className={`w-3.5 h-3.5 transition-transform ${expanded ? "rotate-180" : ""}`} fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {expanded && (
        <div className="px-3 pb-3">
          {profiles.length === 0 && !showCreate && (
            <p className="text-[11px] text-slate-400 dark:text-gray-500 px-2 py-2">{messages.no_profiles}</p>
          )}

          {profiles.map((profile) => (
            <div key={profile.name} className="px-2 py-2 rounded-lg hover:bg-slate-50 dark:hover:bg-gray-700/30">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-slate-700 dark:text-gray-200">{profile.name}</span>
                <div className="flex items-center gap-1">
                  <button
                    onClick={() => handleActivate(profile.name)}
                    className="text-[10px] px-2 py-0.5 rounded bg-emerald-100 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400 hover:bg-emerald-200 dark:hover:bg-emerald-900/50 transition-colors"
                  >
                    {messages.activate}
                  </button>
                  <button
                    onClick={() => handleDeactivate(profile.name)}
                    className="text-[10px] px-2 py-0.5 rounded bg-slate-100 dark:bg-gray-700 text-slate-500 dark:text-gray-400 hover:bg-slate-200 dark:hover:bg-gray-600 transition-colors"
                  >
                    {messages.deactivate}
                  </button>
                  <button
                    onClick={() => handleDelete(profile.name)}
                    className="text-[10px] px-1.5 py-0.5 rounded text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30 hover:text-red-500 transition-colors"
                  >
                    &times;
                  </button>
                </div>
              </div>
              {profile.group_names.length > 0 && (
                <div className="flex flex-wrap gap-1 mt-1">
                  {profile.group_names.map((g) => (
                    <span key={g} className="text-[10px] px-1.5 py-0.5 rounded bg-slate-100 dark:bg-gray-700 text-slate-500 dark:text-gray-400">
                      {g}
                    </span>
                  ))}
                </div>
              )}
            </div>
          ))}

          {showCreate ? (
            <form onSubmit={handleCreate} className="mt-2 px-2 space-y-2">
              <input
                type="text"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                placeholder={messages.profile_name}
                autoFocus
                className="w-full px-2 py-1.5 text-xs border border-slate-200 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-lg text-slate-800 dark:text-gray-200 placeholder:text-slate-300"
              />
              <input
                type="text"
                value={newGroups}
                onChange={(e) => setNewGroups(e.target.value)}
                placeholder={`${messages.profile_groups} (group1, group2)`}
                className="w-full px-2 py-1.5 text-xs border border-slate-200 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-lg text-slate-800 dark:text-gray-200 placeholder:text-slate-300"
              />
              <div className="flex gap-2">
                <button type="submit" className="text-[11px] px-3 py-1 bg-indigo-500 text-white rounded-md hover:bg-indigo-600 transition-colors">
                  {messages.create}
                </button>
                <button type="button" onClick={() => setShowCreate(false)} className="text-[11px] px-3 py-1 text-slate-500 hover:bg-slate-100 dark:hover:bg-gray-700 rounded-md transition-colors">
                  {messages.cancel}
                </button>
              </div>
            </form>
          ) : (
            <button
              onClick={() => setShowCreate(true)}
              className="mt-1 w-full text-[11px] text-indigo-500 hover:text-indigo-600 dark:text-indigo-400 py-1.5 rounded-lg hover:bg-indigo-50 dark:hover:bg-indigo-900/20 transition-colors"
            >
              + {messages.create_profile}
            </button>
          )}
        </div>
      )}
    </div>
  );
});
