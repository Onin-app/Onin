/**
 * 插件设置项类型定义
 */

export interface SettingOption {
  label: string;
  value: string;
}

// 所有字段类型的公共属性
interface BaseSettingField {
  key: string;                    // 设置项的唯一标识
  label: string;                  // 显示标签
  description?: string;           // 描述信息
  required?: boolean;             // 是否必填
}

// 文本输入框
export interface TextSettingField extends BaseSettingField {
  type: 'text';
  defaultValue?: string;
  placeholder?: string;
  maxLength?: number;
  minLength?: number;
}

// 密码输入框
export interface PasswordSettingField extends BaseSettingField {
  type: 'password';
  defaultValue?: string;
  placeholder?: string;
  maxLength?: number;
  minLength?: number;
}

// 多行文本
export interface TextareaSettingField extends BaseSettingField {
  type: 'textarea';
  defaultValue?: string;
  placeholder?: string;
  maxLength?: number;
  minLength?: number;
}

// 数字输入框
export interface NumberSettingField extends BaseSettingField {
  type: 'number';
  defaultValue?: number;
  placeholder?: string;
  min?: number;
  max?: number;
  step?: number;
}

// 颜色选择器
export interface ColorSettingField extends BaseSettingField {
  type: 'color';
  defaultValue?: string;
}

// 日期选择器
export interface DateSettingField extends BaseSettingField {
  type: 'date';
  defaultValue?: string;
}

// 时间选择器
export interface TimeSettingField extends BaseSettingField {
  type: 'time';
  defaultValue?: string;
}

// 日期时间选择器
export interface DatetimeSettingField extends BaseSettingField {
  type: 'datetime';
  defaultValue?: string;
}

// 滑块
export interface SliderSettingField extends BaseSettingField {
  type: 'slider';
  defaultValue?: number;
  min?: number;
  max?: number;
  step?: number;
}

// 开关
export interface SwitchSettingField extends BaseSettingField {
  type: 'switch';
  defaultValue?: boolean;
}

// 单选按钮组
export interface RadioSettingField extends BaseSettingField {
  type: 'radio';
  defaultValue?: string;
  options: SettingOption[];
}

// 下拉选择（单选）
export interface SelectSettingField extends BaseSettingField {
  type: 'select';
  defaultValue?: string;
  placeholder?: string;
  options: SettingOption[];
  multiple?: false;
}

// 下拉选择（多选）
export interface MultiSelectSettingField extends BaseSettingField {
  type: 'select';
  defaultValue?: string[];
  placeholder?: string;
  options: SettingOption[];
  multiple: true;
}

// 按钮
export interface ButtonSettingField extends BaseSettingField {
  type: 'button';
  buttonText?: string;            // 按钮显示文本
  onClick?: () => void;           // 按钮点击事件
}

// 判别联合类型
export type SettingField =
  | TextSettingField
  | PasswordSettingField
  | TextareaSettingField
  | NumberSettingField
  | ColorSettingField
  | DateSettingField
  | TimeSettingField
  | DatetimeSettingField
  | SliderSettingField
  | SwitchSettingField
  | RadioSettingField
  | SelectSettingField
  | MultiSelectSettingField
  | ButtonSettingField;

export interface PluginSettingsSchema {
  fields: SettingField[];
}

export type PluginSettingsValues = Record<string, string | number | boolean | string[] | null | undefined>;
