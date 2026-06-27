import { Button, InputNumber, Select, Space, Typography } from "antd";
import {
  ArrowDownOutlined,
  ArrowUpOutlined,
  DeleteOutlined,
  PlusOutlined,
} from "@ant-design/icons";
import type { Action, MouseButton } from "../types/config";
import { defaultAction } from "../types/config";
import { HotkeyInput, formatHotkey } from "./HotkeyInput";

const mouseButtonOptions: { label: string; value: MouseButton }[] = [
  { label: "左键", value: "left" },
  { label: "右键", value: "right" },
  { label: "中键", value: "middle" },
];

export interface ActionEditorProps {
  value?: Action[];
  onChange?: (value: Action[]) => void;
}

export function ActionEditor({ value = [], onChange }: ActionEditorProps) {
  const update = (next: Action[]) => onChange?.(next);

  const updateAt = (index: number, action: Action) => {
    const next = [...value];
    next[index] = action;
    update(next);
  };

  const move = (index: number, direction: -1 | 1) => {
    const target = index + direction;
    if (target < 0 || target >= value.length) return;
    const next = [...value];
    [next[index], next[target]] = [next[target], next[index]];
    update(next);
  };

  return (
    <div className="action-editor">
      <Space direction="vertical" size={10} className="full-width">
        {value.map((action, index) => (
          <div className="action-row" key={index}>
            <Typography.Text className="action-index">{index + 1}</Typography.Text>
            <Select
              className="action-type"
              value={action.type}
              options={[
                { label: "键盘组合键", value: "keyCombo" },
                { label: "鼠标点击", value: "mouseClick" },
                { label: "等待", value: "delay" },
              ]}
              onChange={(type) => updateAt(index, createActionForType(type))}
            />
            <div className="action-main">{renderActionMain(action, (next) => updateAt(index, next))}</div>
            {action.type !== "delay" ? (
              <InputNumber
                className="delay-after"
                min={0}
                max={60000}
                addonAfter="ms"
                value={action.delayAfterMs ?? 0}
                onChange={(delayAfterMs) =>
                  updateAt(index, { ...action, delayAfterMs: Number(delayAfterMs ?? 0) } as Action)
                }
              />
            ) : null}
            <Space.Compact>
              <Button
                aria-label="上移动作"
                icon={<ArrowUpOutlined />}
                disabled={index === 0}
                onClick={() => move(index, -1)}
              />
              <Button
                aria-label="下移动作"
                icon={<ArrowDownOutlined />}
                disabled={index === value.length - 1}
                onClick={() => move(index, 1)}
              />
              <Button
                danger
                aria-label="删除动作"
                icon={<DeleteOutlined />}
                onClick={() => update(value.filter((_, itemIndex) => itemIndex !== index))}
              />
            </Space.Compact>
          </div>
        ))}
        <Button
          icon={<PlusOutlined />}
          onClick={() => update([...value, defaultAction()])}
        >
          添加动作
        </Button>
      </Space>
    </div>
  );
}

export function summarizeActions(actions: Action[]): string {
  if (actions.length === 0) return "未配置动作";
  return actions.map(describeAction).join(" -> ");
}

function renderActionMain(action: Action, onChange: (value: Action) => void) {
  if (action.type === "keyCombo") {
    return (
      <HotkeyInput
        value={action.keys}
        onChange={(keys) => {
          if (keys) onChange({ ...action, keys });
        }}
      />
    );
  }

  if (action.type === "mouseClick") {
    return (
      <Space>
        <Select
          className="mouse-button-select"
          value={action.button}
          options={mouseButtonOptions}
          onChange={(button) => onChange({ ...action, button })}
        />
        <Select
          className="click-count-select"
          value={action.clickCount ?? 1}
          options={[
            { label: "单击", value: 1 },
            { label: "双击", value: 2 },
          ]}
          onChange={(clickCount) => onChange({ ...action, clickCount })}
        />
      </Space>
    );
  }

  return (
    <InputNumber
      min={1}
      max={60000}
      addonAfter="ms"
      value={action.durationMs}
      onChange={(durationMs) => onChange({ ...action, durationMs: Number(durationMs ?? 1) })}
    />
  );
}

function createActionForType(type: Action["type"]): Action {
  if (type === "mouseClick") {
    return { type, button: "left", clickCount: 1, delayAfterMs: 0 };
  }
  if (type === "delay") {
    return { type, durationMs: 100 };
  }
  return defaultAction();
}

function describeAction(action: Action): string {
  if (action.type === "keyCombo") {
    return formatHotkey(action.keys);
  }
  if (action.type === "mouseClick") {
    const button = action.button === "left" ? "左键" : action.button === "right" ? "右键" : "中键";
    return `${button}${(action.clickCount ?? 1) === 2 ? "双击" : "单击"}`;
  }
  return `等待 ${action.durationMs}ms`;
}
