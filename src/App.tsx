import { useEffect, useMemo, useState } from "react";
import {
  Alert,
  Button,
  Card,
  Empty,
  Layout,
  Modal,
  Space,
  Switch,
  Table,
  Tag,
  Typography,
  message,
} from "antd";
import type { ColumnsType } from "antd/es/table";
import {
  DeleteOutlined,
  DownloadOutlined,
  EditOutlined,
  ExportOutlined,
  ImportOutlined,
  PlusOutlined,
  PoweroffOutlined,
} from "@ant-design/icons";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, save } from "@tauri-apps/plugin-dialog";
import "./App.css";
import {
  createConfig,
  deleteConfig,
  exitApp,
  exportConfig,
  getConfig,
  getRuntimeStatus,
  hideMainWindow,
  importConfig,
  setConfigEnabled,
  setGlobalEnabled,
  setGlobalToggleHotkey,
  updateConfig,
} from "./api/tauri";
import { summarizeActions } from "./components/ActionEditor";
import { ConfigEditorModal } from "./components/ConfigEditorModal";
import { HotkeyInput, HotkeyTag } from "./components/HotkeyInput";
import type {
  AppConfig,
  ImportMode,
  MacroConfig,
  MacroConfigInput,
  RuntimeStatus,
} from "./types/config";

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [editing, setEditing] = useState<MacroConfig | null>(null);
  const [editorOpen, setEditorOpen] = useState(false);

  const enabledCount = useMemo(
    () => config?.configs.filter((item) => item.enabled).length ?? 0,
    [config],
  );

  useEffect(() => {
    void reload();

    let disposed = false;
    const unlisteners: Array<() => void> = [];
    const track = (promise: Promise<() => void>) => {
      void promise.then((unlisten) => {
        if (disposed) {
          unlisten();
        } else {
          unlisteners.push(unlisten);
        }
      });
    };

    track(listen<AppConfig>("config://changed", (event) => setConfig(event.payload)));
    track(listen<RuntimeStatus>("runtime://changed", (event) => setRuntime(event.payload)));
    track(listen<{ message: string }>("config://load_failed", (event) => {
      message.warning(event.payload.message);
    }));
    track(listen<{ message: string }>("hotkey://register_failed", (event) => {
      message.warning(event.payload.message);
      void refreshRuntime();
    }));
    track(listen<{ message?: string }>("action://failed", (event) => {
      if (event.payload.message && event.payload.message !== "动作已取消") {
        message.error(event.payload.message);
      }
    }));

    const appWindow = getCurrentWindow();
    track(appWindow.onCloseRequested((event) => {
      event.preventDefault();
      void exitApp();
    }));
    track(appWindow.onResized(async () => {
      if (await appWindow.isMinimized()) {
        await hideMainWindow();
      }
    }));

    return () => {
      disposed = true;
      for (const unlisten of unlisteners) unlisten();
    };
  }, []);

  async function reload() {
    setLoading(true);
    try {
      const [nextConfig, nextRuntime] = await Promise.all([getConfig(), getRuntimeStatus()]);
      setConfig(nextConfig);
      setRuntime(nextRuntime);
    } catch (error) {
      message.error(String(error));
    } finally {
      setLoading(false);
    }
  }

  async function refreshRuntime() {
    try {
      setRuntime(await getRuntimeStatus());
    } catch {
      // Runtime status is auxiliary UI state; ignore transient refresh failures.
    }
  }

  async function handleSubmit(input: MacroConfigInput) {
    setSaving(true);
    try {
      const next = editing
        ? await updateConfig(editing.id, input)
        : await createConfig(input);
      setConfig(next);
      setEditorOpen(false);
      setEditing(null);
      await refreshRuntime();
    } catch (error) {
      message.error(String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleSetConfigEnabled(item: MacroConfig, enabled: boolean) {
    try {
      setConfig(await setConfigEnabled(item.id, enabled));
      await refreshRuntime();
    } catch (error) {
      message.error(String(error));
    }
  }

  async function handleDelete(item: MacroConfig) {
    Modal.confirm({
      title: "删除配置",
      content: `确定删除「${item.name}」吗？`,
      okText: "删除",
      okButtonProps: { danger: true },
      cancelText: "取消",
      onOk: async () => {
        try {
          setConfig(await deleteConfig(item.id));
          await refreshRuntime();
        } catch (error) {
          message.error(String(error));
        }
      },
    });
  }

  async function handleImport(mode: ImportMode) {
    const selected = await open({
      title: "导入配置",
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!selected || Array.isArray(selected)) return;

    try {
      setConfig(await importConfig(selected, mode));
      await refreshRuntime();
      message.success("配置已导入");
    } catch (error) {
      message.error(String(error));
    }
  }

  async function handleExport() {
    const target = await save({
      title: "导出配置",
      defaultPath: "wudi-xiaolingdang-config.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!target) return;

    try {
      await exportConfig(target);
      message.success("配置已导出");
    } catch (error) {
      message.error(String(error));
    }
  }

  const columns: ColumnsType<MacroConfig> = [
    {
      title: "配置",
      dataIndex: "name",
      width: 180,
      render: (_, item) => (
        <Space direction="vertical" size={2}>
          <Typography.Text strong>{item.name}</Typography.Text>
          {runtime?.configErrors[item.id] ? (
            <Typography.Text type="danger" className="table-subtext">
              {runtime.configErrors[item.id]}
            </Typography.Text>
          ) : null}
        </Space>
      ),
    },
    {
      title: "触发快捷键",
      dataIndex: "triggerHotkey",
      width: 150,
      render: (_, item) => <HotkeyTag hotkey={item.triggerHotkey} />,
    },
    {
      title: "启停快捷键",
      dataIndex: "toggleHotkey",
      width: 150,
      render: (_, item) => <HotkeyTag hotkey={item.toggleHotkey} />,
    },
    {
      title: "动作",
      dataIndex: "actions",
      render: (_, item) => <Typography.Text>{summarizeActions(item.actions)}</Typography.Text>,
    },
    {
      title: "状态",
      width: 90,
      render: (_, item) => (
        <Tag color={item.enabled ? "green" : "default"}>{item.enabled ? "开启" : "关闭"}</Tag>
      ),
    },
    {
      title: "启用",
      width: 90,
      render: (_, item) => (
        <Switch
          size="small"
          checked={item.enabled}
          onChange={(checked) => void handleSetConfigEnabled(item, checked)}
        />
      ),
    },
    {
      title: "操作",
      width: 116,
      render: (_, item) => (
        <Space.Compact>
          <Button
            aria-label="编辑配置"
            icon={<EditOutlined />}
            onClick={() => {
              setEditing(item);
              setEditorOpen(true);
            }}
          />
          <Button
            danger
            aria-label="删除配置"
            icon={<DeleteOutlined />}
            onClick={() => void handleDelete(item)}
          />
        </Space.Compact>
      ),
    },
  ];

  return (
    <Layout className="app-shell">
      <Layout.Content className="app-content">
        <Space direction="vertical" size={16} className="full-width">
          <Card className="toolbar-card" variant="borderless">
            <div className="toolbar">
              <div>
                <Typography.Title level={3} className="app-title">
                  无敌小铃铛
                </Typography.Title>
                <Typography.Text type="secondary">
                  {enabledCount} 个配置已开启，共 {config?.configs.length ?? 0} 个配置
                </Typography.Text>
              </div>
              <Space wrap>
                <Space>
                  <PoweroffOutlined />
                  <Switch
                    checked={config?.globalEnabled ?? false}
                    checkedChildren="全局开启"
                    unCheckedChildren="全局关闭"
                    onChange={async (checked) => {
                      try {
                        setConfig(await setGlobalEnabled(checked));
                        await refreshRuntime();
                      } catch (error) {
                        message.error(String(error));
                      }
                    }}
                  />
                </Space>
                <HotkeyInput
                  allowClear
                  placeholder="全局启停快捷键"
                  value={config?.globalToggleHotkey ?? null}
                  onChange={async (hotkey) => {
                    try {
                      setConfig(await setGlobalToggleHotkey(hotkey));
                      await refreshRuntime();
                    } catch (error) {
                      message.error(String(error));
                    }
                  }}
                />
                <Button
                  icon={<PlusOutlined />}
                  type="primary"
                  onClick={() => {
                    setEditing(null);
                    setEditorOpen(true);
                  }}
                >
                  新建配置
                </Button>
                <Button icon={<ImportOutlined />} onClick={() => void handleImport("replace")}>
                  覆盖导入
                </Button>
                <Button icon={<DownloadOutlined />} onClick={() => void handleImport("append")}>
                  追加导入
                </Button>
                <Button icon={<ExportOutlined />} onClick={() => void handleExport()}>
                  导出
                </Button>
              </Space>
            </div>
          </Card>

          {runtime?.globalError ? (
            <Alert type="warning" showIcon message={runtime.globalError} />
          ) : null}

          <Card className="table-card" variant="borderless">
            <Table
              rowKey="id"
              loading={loading}
              columns={columns}
              dataSource={config?.configs ?? []}
              pagination={false}
              locale={{
                emptyText: (
                  <Empty description="还没有配置">
                    <Button
                      type="primary"
                      icon={<PlusOutlined />}
                      onClick={() => {
                        setEditing(null);
                        setEditorOpen(true);
                      }}
                    >
                      新建配置
                    </Button>
                  </Empty>
                ),
              }}
            />
          </Card>
        </Space>
      </Layout.Content>
      <ConfigEditorModal
        open={editorOpen}
        config={editing}
        saving={saving}
        onCancel={() => {
          setEditorOpen(false);
          setEditing(null);
        }}
        onSubmit={handleSubmit}
      />
    </Layout>
  );
}

export default App;
