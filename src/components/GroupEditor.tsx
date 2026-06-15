import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface VariableInfo {
  key: string;
  value: string;
  path_mode: string;
}

interface GroupDetail {
  name: string;
  description: string;
  active: boolean;
  priority: number;
  variables: VariableInfo[];
}

interface Props {
  groupName: string;
  onUpdate: () => void;
}

export function GroupEditor({ groupName, onUpdate }: Props) {
  const [detail, setDetail] = useState<GroupDetail | null>(null);
  const [newKey, setNewKey] = useState("");
  const [newValue, setNewValue] = useState("");
  const [newMode, setNewMode] = useState("override");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadDetail();
  }, [groupName]);

  async function loadDetail() {
    try {
      const result = await invoke<GroupDetail>("get_group_detail", { name: groupName });
      setDetail(result);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleAddVariable(e: React.FormEvent) {
    e.preventDefault();
    if (!newKey.trim()) return;

    try {
      await invoke("set_variable", {
        groupName,
        key: newKey.trim(),
        value: newValue,
        pathMode: newMode,
      });
      setNewKey("");
      setNewValue("");
      setNewMode("override");
      await loadDetail();
      onUpdate();
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleRemoveVariable(key: string) {
    try {
      await invoke("remove_variable", { groupName, key });
      await loadDetail();
      onUpdate();
    } catch (err) {
      setError(String(err));
    }
  }

  if (!detail) {
    return <div className="p-6 text-gray-400">Loading...</div>;
  }

  return (
    <div className="p-6">
      {/* Group Header */}
      <div className="mb-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white">{detail.name}</h2>
        <p className="text-sm text-gray-500 mt-1">
          {detail.description || "No description"} &middot; Priority: {detail.priority} &middot;{" "}
          <span className={detail.active ? "text-green-600" : "text-gray-400"}>
            {detail.active ? "Active" : "Inactive"}
          </span>
        </p>
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded text-sm text-red-700">
          {error}
        </div>
      )}

      {/* Variables Table */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-gray-50 dark:bg-gray-750 border-b border-gray-200 dark:border-gray-700">
            <tr>
              <th className="text-left px-4 py-2 font-medium text-gray-600">Key</th>
              <th className="text-left px-4 py-2 font-medium text-gray-600">Value</th>
              <th className="text-left px-4 py-2 font-medium text-gray-600 w-24">Mode</th>
              <th className="w-16"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100 dark:divide-gray-700">
            {detail.variables.map((v) => (
              <tr key={v.key} className="hover:bg-gray-50 dark:hover:bg-gray-750">
                <td className="px-4 py-2 font-mono text-xs text-blue-700 dark:text-blue-400">
                  {v.key}
                </td>
                <td className="px-4 py-2 font-mono text-xs text-gray-700 dark:text-gray-300 break-all max-w-md">
                  {v.value}
                </td>
                <td className="px-4 py-2">
                  <span
                    className={`text-xs px-2 py-0.5 rounded ${
                      v.path_mode === "prepend"
                        ? "bg-purple-100 text-purple-700"
                        : v.path_mode === "append"
                        ? "bg-orange-100 text-orange-700"
                        : "bg-gray-100 text-gray-600"
                    }`}
                  >
                    {v.path_mode}
                  </span>
                </td>
                <td className="px-4 py-2 text-right">
                  <button
                    onClick={() => handleRemoveVariable(v.key)}
                    className="text-red-400 hover:text-red-600 text-xs"
                  >
                    Remove
                  </button>
                </td>
              </tr>
            ))}
            {detail.variables.length === 0 && (
              <tr>
                <td colSpan={4} className="px-4 py-6 text-center text-gray-400">
                  No variables. Add one below.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {/* Add Variable Form */}
      <form onSubmit={handleAddVariable} className="mt-4 flex gap-2 items-end">
        <div className="flex-1">
          <label className="block text-xs text-gray-500 mb-1">Key</label>
          <input
            type="text"
            value={newKey}
            onChange={(e) => setNewKey(e.target.value)}
            placeholder="VARIABLE_NAME"
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>
        <div className="flex-[2]">
          <label className="block text-xs text-gray-500 mb-1">Value</label>
          <input
            type="text"
            value={newValue}
            onChange={(e) => setNewValue(e.target.value)}
            placeholder="/path/to/something"
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>
        <div className="w-28">
          <label className="block text-xs text-gray-500 mb-1">Mode</label>
          <select
            value={newMode}
            onChange={(e) => setNewMode(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500"
          >
            <option value="override">override</option>
            <option value="prepend">prepend</option>
            <option value="append">append</option>
          </select>
        </div>
        <button
          type="submit"
          className="px-4 py-2 bg-green-600 text-white text-sm rounded-lg hover:bg-green-700 transition-colors"
        >
          Add
        </button>
      </form>
    </div>
  );
}
