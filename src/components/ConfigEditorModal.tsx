import { Form, Input, Modal, Switch } from "antd";
import { useEffect } from "react";
import type { MacroConfig, MacroConfigInput } from "../types/config";
import { defaultAction, defaultHotkey } from "../types/config";
import { ActionEditor } from "./ActionEditor";
import { HotkeyInput } from "./HotkeyInput";

export interface ConfigEditorModalProps {
  open: boolean;
  config?: MacroConfig | null;
  saving?: boolean;
  onCancel: () => void;
  onSubmit: (input: MacroConfigInput) => Promise<void> | void;
}

export function ConfigEditorModal({
  open,
  config,
  saving = false,
  onCancel,
  onSubmit,
}: ConfigEditorModalProps) {
  const [form] = Form.useForm<MacroConfigInput>();

  useEffect(() => {
    if (!open) return;
    form.setFieldsValue(
      config
        ? {
            name: config.name,
            enabled: config.enabled,
            triggerHotkey: config.triggerHotkey,
            toggleHotkey: config.toggleHotkey ?? null,
            actions: config.actions,
          }
        : {
            name: "",
            enabled: true,
            triggerHotkey: defaultHotkey(),
            toggleHotkey: null,
            actions: [defaultAction()],
          },
    );
  }, [config, form, open]);

  return (
    <Modal
      title={config ? "编辑配置" : "新建配置"}
      open={open}
      confirmLoading={saving}
      width={860}
      onCancel={onCancel}
      onOk={() => form.submit()}
      destroyOnHidden
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={(values) => onSubmit(values)}
      >
        <div className="config-form-grid">
          <Form.Item
            name="name"
            label="配置名称"
            rules={[{ required: true, whitespace: true, message: "请输入配置名称" }]}
          >
            <Input placeholder="例如：铃铛快捷操作" maxLength={40} />
          </Form.Item>
          <Form.Item name="enabled" label="启用" valuePropName="checked">
            <Switch />
          </Form.Item>
        </div>
        <div className="config-form-grid">
          <Form.Item
            name="triggerHotkey"
            label="触发快捷键"
            rules={[{ required: true, message: "请设置触发快捷键" }]}
          >
            <HotkeyInput />
          </Form.Item>
          <Form.Item name="toggleHotkey" label="配置启停快捷键">
            <HotkeyInput allowClear placeholder="可选" />
          </Form.Item>
        </div>
        <Form.Item
          name="actions"
          label="动作序列"
          rules={[
            {
              validator: (_, value) =>
                Array.isArray(value) && value.length > 0
                  ? Promise.resolve()
                  : Promise.reject(new Error("至少需要一个动作")),
            },
          ]}
        >
          <ActionEditor />
        </Form.Item>
      </Form>
    </Modal>
  );
}
