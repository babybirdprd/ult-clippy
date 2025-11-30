import React, { useEffect, useState, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Database from '@tauri-apps/plugin-sql';
import { onClipboardUpdate, readText, readImageBase64 } from 'tauri-plugin-clipboard-api';
import {
  Search,
  Settings,
  Clock,
  Pin,
  Copy,
  Image as ImageIcon,
  Type as TextIcon,
  Edit2,
  Trash2,
  Tag
} from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

// --- Utility ---
function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// --- Types ---
interface ClipboardItem {
  id: number;
  content: string; // Text content or Base64 Image
  type: 'text' | 'image' | 'html';
  is_pinned: boolean;
  created_at: string;
  name: string | null;
  category: string | null;
}

// --- DB Helper ---
const DB_NAME = 'sqlite:clipboard.db';
const initDb = async () => {
  const db = await Database.load(DB_NAME);
  await db.execute(`
    CREATE TABLE IF NOT EXISTS clipboard_history (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      content TEXT NOT NULL,
      type TEXT DEFAULT 'text',
      is_pinned BOOLEAN DEFAULT 0,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
      name TEXT,
      category TEXT
    );
  `);
  return db;
};

// --- Components ---

const Card = ({ item, onClick, onPin, onEdit, onDelete }: {
  item: ClipboardItem,
  onClick: (item: ClipboardItem) => void,
  onPin: (e: React.MouseEvent) => void,
  onEdit: (e: React.MouseEvent) => void,
  onDelete: (e: React.MouseEvent) => void
}) => {

  const isColor = item.type === 'text' && /^#[0-9A-F]{6}$/i.test(item.content.trim());
  const displayTitle = item.name || (isColor ? item.content : (item.type === 'image' ? 'Image Clip' : 'Saved Text'));

  // Dynamic header color based on category or type
  const headerColors: Record<string, string> = {
    'work': 'bg-blue-600',
    'personal': 'bg-green-600',
    'design': 'bg-purple-600',
    'code': 'bg-orange-600',
    'default': 'bg-zinc-700'
  };
  const headerColor = item.category ? (headerColors[item.category.toLowerCase()] || headerColors['default']) : headerColors['default'];

  return (
    <div
      className="bg-zinc-800 rounded-xl overflow-hidden cursor-pointer hover:ring-2 hover:ring-blue-500 transition-all group relative border border-zinc-700"
      onClick={() => onClick(item)}
    >
      {/* Header */}
      <div className={cn("px-4 py-3 flex items-center justify-between", headerColor)}>
        <div className="flex items-center gap-2 text-white/90">
          {item.type === 'image' ? <ImageIcon size={16} /> : <TextIcon size={16} />}
          <span className="font-semibold text-sm truncate max-w-[120px]">{displayTitle}</span>
        </div>
        <div className="flex items-center gap-1">
          {item.is_pinned && <Pin size={14} className="fill-white text-white" />}
          <span className="text-xs text-white/70">
            {formatDistanceToNow(new Date(item.created_at), { addSuffix: true })}
          </span>
        </div>
      </div>

      {/* Body */}
      <div className="p-4 h-32 overflow-hidden bg-zinc-900/50 relative">
        {item.type === 'image' ? (
          <img src={item.content.startsWith('data:') ? item.content : `data:image/png;base64,${item.content}`} alt="clip" className="w-full h-full object-cover rounded-md" />
        ) : isColor ? (
           <div className="w-full h-full rounded-md flex items-center justify-center" style={{ backgroundColor: item.content }}>
             <span className="bg-black/50 text-white px-2 py-1 rounded text-sm font-mono">{item.content}</span>
           </div>
        ) : (
          <p className="text-zinc-300 text-sm whitespace-pre-wrap font-mono break-words line-clamp-5">
            {item.content}
          </p>
        )}

        {/* Hover Actions */}
        <div className="absolute top-2 right-2 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity bg-zinc-900/80 p-1 rounded-lg">
          <button
            onClick={(e) => { e.stopPropagation(); onPin(e); }}
            className={cn("p-1.5 rounded-md hover:bg-zinc-700 text-zinc-400 hover:text-white", item.is_pinned && "text-blue-400")}
            title="Pin"
          >
            <Pin size={14} className={cn(item.is_pinned && "fill-current")} />
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); onEdit(e); }}
            className="p-1.5 rounded-md hover:bg-zinc-700 text-zinc-400 hover:text-white"
            title="Edit"
          >
            <Edit2 size={14} />
          </button>
           <button
            onClick={(e) => { e.stopPropagation(); onDelete(e); }}
            className="p-1.5 rounded-md hover:bg-red-900/50 text-zinc-400 hover:text-red-400"
            title="Delete"
          >
            <Trash2 size={14} />
          </button>
        </div>
      </div>

      {/* Category Tag if exists */}
      {item.category && (
        <div className="px-4 py-2 bg-zinc-900 border-t border-zinc-700 flex items-center gap-1">
          <Tag size={12} className="text-zinc-500" />
          <span className="text-xs text-zinc-400 capitalize">{item.category}</span>
        </div>
      )}
    </div>
  );
};

const EditModal = ({ item, onClose, onSave }: { item: ClipboardItem, onClose: () => void, onSave: (id: number, name: string, category: string) => void }) => {
  const [name, setName] = useState(item.name || '');
  const [category, setCategory] = useState(item.category || '');

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" onClick={onClose}>
      <div className="bg-zinc-900 border border-zinc-700 rounded-xl p-6 w-full max-w-md shadow-2xl" onClick={e => e.stopPropagation()}>
        <h3 className="text-xl font-bold mb-4">Edit Clip</h3>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-zinc-400 mb-1">Name</label>
            <input
              type="text"
              value={name}
              onChange={e => setName(e.target.value)}
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="Give it a name..."
            />
          </div>
          <div>
            <label className="block text-sm text-zinc-400 mb-1">Category</label>
            <input
              type="text"
              value={category}
              onChange={e => setCategory(e.target.value)}
              className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="work, personal, design..."
            />
          </div>
        </div>

        <div className="flex justify-end gap-3 mt-6">
          <button onClick={onClose} className="px-4 py-2 text-zinc-400 hover:text-white">Cancel</button>
          <button
            onClick={() => onSave(item.id, name, category)}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium"
          >
            Save Changes
          </button>
        </div>
      </div>
    </div>
  );
};

// --- Main App ---

export default function App() {
  const [items, setItems] = useState<ClipboardItem[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [activeTab, setActiveTab] = useState<'history' | 'pinned'>('history');
  const [editingItem, setEditingItem] = useState<ClipboardItem | null>(null);

  // Load Data
  const refreshList = async () => {
    try {
      const db = await Database.load(DB_NAME);
      const rows = await db.select<ClipboardItem[]>(
        `SELECT * FROM clipboard_history ORDER BY created_at DESC LIMIT 100`
      );
      setItems(rows);
    } catch (error) {
      console.error("Failed to refresh list:", error);
    }
  };

  // Init
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setup = async () => {
      try {
        const db = await initDb();
        await refreshList();

        // Listen for clipboard changes
        // Note: CrossCopy's onClipboardUpdate might trigger multiple times or for self.
        // In a real app, we need to handle "ignore self" if we write to clipboard.
        unlisten = await onClipboardUpdate(async () => {
          // Check what's in clipboard
          // Try text first
          let content = '';
          let type: 'text' | 'image' = 'text';

          try {
             // Try reading text
             const text = await readText();
             if (text && text.trim().length > 0) {
               content = text;
               type = 'text';
             } else {
               // Try reading image
               // readImageBase64 returns Base64 string usually
               const img = await readImageBase64();
               if (img) {
                 content = img; // usually base64
                 type = 'image';
               }
             }
          } catch (e) {
             console.error("Error reading clipboard:", e);
          }

          if (content) {
            // Check if duplicate (top of stack)
            const top = await db.select<ClipboardItem[]>('SELECT content FROM clipboard_history ORDER BY id DESC LIMIT 1');
            if (top.length > 0 && top[0].content === content) {
              return; // Duplicate
            }

            await db.execute(
              'INSERT INTO clipboard_history (content, type) VALUES ($1, $2)',
              [content, type]
            );
            refreshList();
          }
        });
      } catch (e) {
        console.error("Setup failed:", e);
      }
    };

    setup();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handlePaste = async (item: ClipboardItem) => {
    try {
      // invoke backend to paste
      // First write to clipboard
      // We assume the user wants to paste this item
      // Note: Writing to clipboard will trigger our listener.
      // Ideally we should have a flag to ignore next update.
      // But for MVP, the "Duplicate check" above handles it if content is same.

      // Write content back to OS clipboard
      // We need writeText or writeImage
      // But tauri-plugin-clipboard-api has these.
      // Actually, we can just use the plugin's write functions.
      // But let's check `tauri-plugin-clipboard-api` exports.
      // It exports `writeText`, `writeImage`.

      const { writeText, writeImageBase64 } = await import('tauri-plugin-clipboard-api');

      if (item.type === 'image') {
        await writeImageBase64(item.content);
      } else {
        await writeText(item.content);
      }

      await invoke('paste_selection');
    } catch (e) {
      console.error("Paste failed:", e);
    }
  };

  const handlePin = async (e: React.MouseEvent, item: ClipboardItem) => {
    e.stopPropagation();
    const db = await Database.load(DB_NAME);
    await db.execute('UPDATE clipboard_history SET is_pinned = $1 WHERE id = $2', [!item.is_pinned, item.id]);
    refreshList();
  };

  const handleDelete = async (e: React.MouseEvent, item: ClipboardItem) => {
    e.stopPropagation();
    if (!confirm("Delete this clip?")) return;
    const db = await Database.load(DB_NAME);
    await db.execute('DELETE FROM clipboard_history WHERE id = $1', [item.id]);
    refreshList();
  };

  const handleSaveEdit = async (id: number, name: string, category: string) => {
    const db = await Database.load(DB_NAME);
    await db.execute('UPDATE clipboard_history SET name = $1, category = $2 WHERE id = $3', [name, category, id]);
    setEditingItem(null);
    refreshList();
  };

  // Filtering
  const filteredItems = useMemo(() => {
    let list = items;

    // 1. Tab Filter
    if (activeTab === 'pinned') {
      list = list.filter(i => i.is_pinned);
    }

    // 2. Search Filter
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      list = list.filter(i =>
        (i.content && i.content.toLowerCase().includes(q)) ||
        (i.name && i.name.toLowerCase().includes(q)) ||
        (i.category && i.category.toLowerCase().includes(q))
      );
    }

    return list;
  }, [items, activeTab, searchQuery]);

  return (
    <div className="h-screen w-full bg-zinc-950 text-white flex flex-col font-sans select-none">

      {/* Top Bar */}
      <div className="flex-none p-4 border-b border-zinc-800 bg-zinc-950/95 backdrop-blur z-10 sticky top-0">
        <div className="flex items-center gap-4 mb-4">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" size={18} />
            <input
              type="text"
              placeholder="Search clips..."
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              className="w-full bg-zinc-900 border border-zinc-800 rounded-xl pl-10 pr-4 py-2.5 text-zinc-200 focus:outline-none focus:ring-2 focus:ring-purple-600 transition-all"
              autoFocus
            />
          </div>
          <button className="p-2 text-zinc-400 hover:text-white hover:bg-zinc-800 rounded-lg transition-colors">
            <Settings size={20} />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex items-center gap-6 border-b border-zinc-800">
          <button
            onClick={() => setActiveTab('history')}
            className={cn(
              "pb-2 text-sm font-medium transition-colors relative",
              activeTab === 'history' ? "text-white" : "text-zinc-500 hover:text-zinc-300"
            )}
          >
            <div className="flex items-center gap-2">
              <Clock size={16} /> Clipboard History
            </div>
            {activeTab === 'history' && <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-purple-600 rounded-t-full" />}
          </button>

          <button
             onClick={() => setActiveTab('pinned')}
             className={cn(
              "pb-2 text-sm font-medium transition-colors relative",
              activeTab === 'pinned' ? "text-white" : "text-zinc-500 hover:text-zinc-300"
            )}
          >
            <div className="flex items-center gap-2">
              <Pin size={16} /> Pinned Clips
            </div>
            {activeTab === 'pinned' && <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-purple-600 rounded-t-full" />}
          </button>
        </div>
      </div>

      {/* Content Area */}
      <div className="flex-1 overflow-y-auto p-4 scrollbar-thin scrollbar-thumb-zinc-800 scrollbar-track-transparent">
        {filteredItems.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-zinc-500 opacity-50">
            <Copy size={48} className="mb-4" />
            <p>No clips found</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 pb-10">
            {filteredItems.map(item => (
              <Card
                key={item.id}
                item={item}
                onClick={() => handlePaste(item)}
                onPin={(e) => handlePin(e, item)}
                onEdit={() => setEditingItem(item)}
                onDelete={(e) => handleDelete(e, item)}
              />
            ))}
          </div>
        )}
      </div>

      {editingItem && (
        <EditModal
          item={editingItem}
          onClose={() => setEditingItem(null)}
          onSave={handleSaveEdit}
        />
      )}
    </div>
  );
}
