import { useState, useEffect, useCallback, memo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "../i18n";

interface VariableInfo {
  key: string;
  value: string;
  path_mode: string;
}

interface HostsEntryInfo {
  ip: string;
  hostname: string;
}

interface GroupDetail {
  name: string;
  description: string;
  kind: string;
  active: boolean;
  priority: number;
  variables: VariableInfo[];
  hosts_entries: HostsEntryInfo[];
}

interface Props {
  groupName: string;
  onUpdate: () => void;
}

export const GroupEditor = memo(function GroupEditor({ groupName, onUpdate }: Props) {
  const { messages } = useI18n();
  const [detail, setDetail] = useState<GroupDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const prevCount = useRef<number>(0);

  // Env form
  const [newKey, setNewKey] = useState("");
  const [newValue, setNewValue] = useState("");
  const [newMode, setNewMode] = useState("override");

  // Hosts form
  const [newIp, setNewIp] = useState("");
  const [newHostname, setNewHostname] = useState("");

  const loadDetail = useCallback(async () => {
    try {
      const result = await invoke<GroupDetail>("get_group_detail", { name: groupName });
      setDetail(result);
      setError(null);
      const count = result.variables.length + result.hosts_entries.length;
      if (count !== prevCount.current) {
        prevCount.current = count;
        onUpdate();
      }
    } catch (e) {
      setError(String(e));
    }
  }, [groupName, onUpdate]);

  useEffect(() => {
    loadDetail();
  }, [loadDetail]);

  const handleAddVariable = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newKey.trim()) return;
    try {
      await invoke("set_variable", { groupName, key: newKey.trim(), value: newValue, pathMode: newMode });
      setNewKey("");
      setNewValue("");
      setNewMode("override");
      await loadDetail();
    } catch (err) {
      setError(String(err));
    }
  }, [groupName, newKey, newValue, newMode, loadDetail]);

  const handleRemoveVariable = useCallback(async (key: string) => {
    try {
      await invoke("remove_variable", { groupName, key });
      await loadDetail();
    } catch (err) {
      setError(String(err));
    }
  }, [groupName, loadDetail]);

  const handleAddHostsEntry = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newIp.trim() || !newHostname.trim()) return;
    try {
      await invoke("add_hosts_entry", { groupName, ip: newIp.trim(), hostname: newHostname.trim() });
      setNewIp("");
      setNewHostname("");
      await loadDetail();
    } catch (err) {
      setError(String(err));
    }
  }, [groupName, newIp, newHostname, loadDetail]);

  const handleRemoveHostsEntry = useCallback(async (hostname: string) => {
    try {
      await invoke("remove_hosts_entry", { groupName, hostname });
      await loadDetail();
    } catch (err) {
      setError(String(err));
    }
  }, [groupName, loadDetail]);

  const handleSyncHosts = useCallback(async () => {
    try {
      await invoke("sync_hosts");
    } catch (err) {
      setError(String(err));
    }
  }, []);

  if (!detail) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="w-5 h-5 border-2 border-indigo-300 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  const isHosts = detail.kind === "hosts";

  return (
    <div className="p-6 max-w-4xl">
      {/* Header */}
      <div className="mb-6">
        <div className="flex items-center gap-3">
          <h2 className="text-xl font-bold text-slate-800 dark:text-white">{detail.name}</h2>
          <span className={`text-[9px] font-bold uppercase px-2 py-0.5 rounded ${
            isHosts
              ? "bg-teal-100 dark:bg-teal-900/30 text-teal-600 dark:text-teal-400"
              : "bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400"
          }`}>
            {isHosts ? "DNS" : "ENV"}
          </span>
          <span className={`text-xs font-medium px-2 py-0.5 rounded-full ${
            detail.active
              ? "bg-emerald-100 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-400"
              : "bg-slate-100 dark:bg-gray-700 text-slate-500 dark:text-gray-400"
          }`}>
            {detail.active ? messages.active : messages.inactive}
          </span>
          {isHosts && detail.active && (
            <button
              onClick={handleSyncHosts}
              className="ml-auto px-3 py-1.5 text-xs font-medium bg-teal-500 hover:bg-teal-600 text-white rounded-lg transition-colors"
            >
              {messages.sync_hosts}
            </button>
          )}
        </div>
        {detail.description && (
          <p className="text-sm text-slate-500 dark:text-gray-400 mt-1">{detail.description}</p>
        )}
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-800 rounded-lg text-sm text-red-600 dark:text-red-400">
          {error}
        </div>
      )}

      {isHosts ? (
        <>
          {/* Hosts Add Form */}
          <form onSubmit={handleAddHostsEntry} className="mb-5">
            <div className="flex gap-2 items-end">
              <div className="w-48">
                <label className="block text-[11px] font-medium text-slate-500 dark:text-gray-400 uppercase tracking-wider mb-1">
                  {messages.ip_address}
                </label>
                <input
                  type="text"
                  value={newIp}
                  onChange={(e) => setNewIp(e.target.value)}
                  placeholder="127.0.0.1"
                  className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-slate-200 dark:border-gray-600 rounded-lg text-sm font-mono placeholder:text-slate-300 dark:placeholder:text-gray-600 text-slate-800 dark:text-gray-200 focus:ring-2 focus:ring-teal-500/20 focus:border-teal-400 transition-shadow"
                />
              </div>
              <div className="flex-1">
                <label className="block text-[11px] font-medium text-slate-500 dark:text-gray-400 uppercase tracking-wider mb-1">
                  {messages.hostname}
                </label>
                <input
                  type="text"
                  value={newHostname}
                  onChange={(e) => setNewHostname(e.target.value)}
                  placeholder="api.local"
                  className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-slate-200 dark:border-gray-600 rounded-lg text-sm font-mono placeholder:text-slate-300 dark:placeholder:text-gray-600 text-slate-800 dark:text-gray-200 focus:ring-2 focus:ring-teal-500/20 focus:border-teal-400 transition-shadow"
                />
              </div>
              <button
                type="submit"
                className="px-4 py-2 bg-teal-500 hover:bg-teal-600 text-white text-sm font-medium rounded-lg transition-colors shadow-sm"
              >
                {messages.add}
              </button>
            </div>
          </form>

          {/* Hosts Table */}
          <div className="bg-white dark:bg-gray-800 rounded-xl border border-slate-200 dark:border-gray-700 overflow-hidden shadow-sm">
            {detail.hosts_entries.length === 0 ? (
              <div className="py-12 text-center">
                <p className="text-sm text-slate-400 dark:text-gray-500">{messages.no_hosts}</p>
                <p className="text-xs text-slate-300 dark:text-gray-600 mt-1">{messages.no_hosts_hint}</p>
              </div>
            ) : (
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-slate-100 dark:border-gray-700 bg-slate-50/50 dark:bg-gray-750">
                    <th className="text-left px-4 py-2.5 text-[11px] font-semibold text-slate-400 dark:text-gray-500 uppercase tracking-wider">{messages.ip_address}</th>
                    <th className="text-left px-4 py-2.5 text-[11px] font-semibold text-slate-400 dark:text-gray-500 uppercase tracking-wider">{messages.hostname}</th>
                    <th className="w-12"></th>
                  </tr>
                </thead>
                <tbody>
                  {detail.hosts_entries.map((entry, i) => (
                    <tr key={entry.hostname} className={`group hover:bg-slate-50/80 dark:hover:bg-gray-700/50 transition-colors ${
                      i < detail.hosts_entries.length - 1 ? "border-b border-slate-50 dark:border-gray-700/50" : ""
                    }`}>
                      <td className="px-4 py-3 font-mono text-xs font-semibold text-teal-600 dark:text-teal-400">{entry.ip}</td>
                      <td className="px-4 py-3 font-mono text-xs text-slate-600 dark:text-gray-300">{entry.hostname}</td>
                      <td className="px-3 py-3">
                        <button
                          onClick={() => handleRemoveHostsEntry(entry.hostname)}
                          className="w-6 h-6 flex items-center justify-center rounded-md opacity-0 group-hover:opacity-100 hover:bg-red-50 dark:hover:bg-red-900/30 text-slate-300 hover:text-red-500 transition-all"
                          title={messages.remove}
                        >
                          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </>
      ) : (
        <>
          {/* Env Add Form */}
          <form onSubmit={handleAddVariable} className="mb-5">
            <div className="flex gap-2 items-end">
              <div className="w-44">
                <label className="block text-[11px] font-medium text-slate-500 dark:text-gray-400 uppercase tracking-wider mb-1">
                  {messages.key}
                </label>
                <input
                  type="text"
                  value={newKey}
                  onChange={(e) => setNewKey(e.target.value)}
                  placeholder="MY_VARIABLE"
                  className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-slate-200 dark:border-gray-600 rounded-lg text-sm font-mono placeholder:text-slate-300 dark:placeholder:text-gray-600 text-slate-800 dark:text-gray-200 focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-400 transition-shadow"
                />
              </div>
              <div className="flex-1">
                <label className="block text-[11px] font-medium text-slate-500 dark:text-gray-400 uppercase tracking-wider mb-1">
                  {messages.value}
                </label>
                <input
                  type="text"
                  value={newValue}
                  onChange={(e) => setNewValue(e.target.value)}
                  placeholder="/usr/local/bin"
                  className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-slate-200 dark:border-gray-600 rounded-lg text-sm font-mono placeholder:text-slate-300 dark:placeholder:text-gray-600 text-slate-800 dark:text-gray-200 focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-400 transition-shadow"
                />
              </div>
              <div className="w-28">
                <label className="block text-[11px] font-medium text-slate-500 dark:text-gray-400 uppercase tracking-wider mb-1">
                  {messages.mode}
                </label>
                <select
                  value={newMode}
                  onChange={(e) => setNewMode(e.target.value)}
                  className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-slate-200 dark:border-gray-600 rounded-lg text-sm text-slate-800 dark:text-gray-200 focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-400 transition-shadow"
                >
                  <option value="override">{messages.mode_override}</option>
                  <option value="prepend">{messages.mode_prepend}</option>
                  <option value="append">{messages.mode_append}</option>
                </select>
              </div>
              <button
                type="submit"
                className="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 text-white text-sm font-medium rounded-lg transition-colors shadow-sm"
              >
                {messages.add}
              </button>
            </div>
          </form>

          {/* Variables Table */}
          <div className="bg-white dark:bg-gray-800 rounded-xl border border-slate-200 dark:border-gray-700 overflow-hidden shadow-sm">
            {detail.variables.length === 0 ? (
              <div className="py-12 text-center">
                <p className="text-sm text-slate-400 dark:text-gray-500">{messages.no_vars}</p>
                <p className="text-xs text-slate-300 dark:text-gray-600 mt-1">{messages.no_vars_hint}</p>
              </div>
            ) : (
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-slate-100 dark:border-gray-700 bg-slate-50/50 dark:bg-gray-750">
                    <th className="text-left px-4 py-2.5 text-[11px] font-semibold text-slate-400 dark:text-gray-500 uppercase tracking-wider">{messages.key}</th>
                    <th className="text-left px-4 py-2.5 text-[11px] font-semibold text-slate-400 dark:text-gray-500 uppercase tracking-wider">{messages.value}</th>
                    <th className="text-left px-4 py-2.5 text-[11px] font-semibold text-slate-400 dark:text-gray-500 uppercase tracking-wider w-24">{messages.mode}</th>
                    <th className="w-12"></th>
                  </tr>
                </thead>
                <tbody>
                  {detail.variables.map((v, i) => (
                    <tr key={v.key} className={`group hover:bg-slate-50/80 dark:hover:bg-gray-700/50 transition-colors ${
                      i < detail.variables.length - 1 ? "border-b border-slate-50 dark:border-gray-700/50" : ""
                    }`}>
                      <td className="px-4 py-3 font-mono text-xs font-semibold text-indigo-600 dark:text-indigo-400">{v.key}</td>
                      <td className="px-4 py-3 font-mono text-xs text-slate-600 dark:text-gray-300 break-all max-w-xs">{v.value}</td>
                      <td className="px-4 py-3">
                        <span className={`text-[10px] font-medium px-2 py-0.5 rounded-full ${
                          v.path_mode === "prepend"
                            ? "bg-violet-100 dark:bg-violet-900/30 text-violet-600 dark:text-violet-400"
                            : v.path_mode === "append"
                            ? "bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400"
                            : "bg-slate-100 dark:bg-gray-700 text-slate-500 dark:text-gray-400"
                        }`}>
                          {v.path_mode === "prepend" ? messages.mode_prepend : v.path_mode === "append" ? messages.mode_append : messages.mode_override}
                        </span>
                      </td>
                      <td className="px-3 py-3">
                        <button
                          onClick={() => handleRemoveVariable(v.key)}
                          className="w-6 h-6 flex items-center justify-center rounded-md opacity-0 group-hover:opacity-100 hover:bg-red-50 dark:hover:bg-red-900/30 text-slate-300 hover:text-red-500 transition-all"
                          title={messages.remove}
                        >
                          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </>
      )}
    </div>
  );
});
