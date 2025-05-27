import React, { useState } from 'react';
import { Modal, Button, Select, Space } from 'antd';

interface Rule {
    field: string;
    operator: string;
    value: any;
}

interface RuleGroup {
    operator: 'and' | 'or';
    rules: (Rule | RuleGroup)[];
}

interface DataField {
    key: string;
    name: string;
    data_type: string;
    allowed_values?: string[];
}

interface DataDictionary {
    fields: DataField[];
}

interface QueryBuilderProps {
    visible: boolean;
    onCancel: () => void;
    onConfirm: (query: any) => void;
    dataDictionary: DataDictionary;
}

const QueryBuilder: React.FC<QueryBuilderProps> = ({ visible, onCancel, onConfirm, dataDictionary }) => {
    const [queryGroup, setQueryGroup] = useState<RuleGroup>({ operator: 'and', rules: [] });

    const getFieldType = (key: string): string =>
        dataDictionary.fields.find(f => f.key === key)?.data_type || 'STRING';

    const getOperators = (type: string): string[] => {
        switch (type.toUpperCase()) {
            case 'NUMBER': return ['=', '!=', '<', '<=', '>', '>=', 'in', 'not in'];
            case 'STRING': return ['=', '!=', 'like', 'not like', 'ilike', 'in', 'not in'];
            case 'BOOLEAN': return ['=', '!=', 'in', 'not in'];
            default: return ['='];
        }
    };

    const renderRule = (rule: Rule, groupPath: number[], ruleIdx: number) => {
        const fieldType = getFieldType(rule.field);
        const operators = getOperators(fieldType);
        const isMulti = ['in', 'not in'].includes(rule.operator);

        const updateRule = (patch: Partial<Rule>) => {
            const updated = { ...queryGroup };
            let cursor: any = updated;
            for (const i of groupPath) cursor = cursor.rules[i];
            cursor.rules[ruleIdx] = { ...cursor.rules[ruleIdx], ...patch };
            setQueryGroup(updated);
        };

        const removeRule = () => {
            const updated = { ...queryGroup };
            let cursor: any = updated;
            for (const i of groupPath) cursor = cursor.rules[i];
            cursor.rules.splice(ruleIdx, 1);
            setQueryGroup(updated);
        };

        return (
            <Space key={ruleIdx} align="start">
                <Select
                    value={rule.field}
                    onChange={val =>
                        updateRule({ field: val, operator: getOperators(getFieldType(val))[0], value: '' })
                    }
                    options={dataDictionary.fields.map(f => ({ label: f.name, value: f.key }))}
                    style={{ width: 160 }}
                />
                <Select
                    value={rule.operator}
                    onChange={val => updateRule({ operator: val, value: '' })}
                    options={operators.map(op => ({ label: op, value: op }))}
                    style={{ width: 120 }}
                />
                <Select
                    mode={isMulti ? 'multiple' : undefined}
                    value={rule.value || []}
                    onChange={val => updateRule({ value: val })}
                    options={
                        dataDictionary.fields.find(f => f.key === rule.field)?.allowed_values?.map(v => ({ label: v, value: v })) || []
                    }
                    style={{ width: 200 }}
                />
                <Button danger onClick={removeRule}>Remove</Button>
            </Space>
        );
    };

    const renderGroup = (group: RuleGroup, groupPath: number[] = []) => {
        return (
            <div style={{ border: '1px solid #ddd', padding: 12, marginBottom: 12 }}>
                <Space direction="vertical" style={{ width: '100%' }}>
                    <Select
                        value={group.operator}
                        onChange={val => {
                            const updated = { ...queryGroup };
                            let cursor: any = updated;
                            for (const i of groupPath) cursor = cursor.rules[i];
                            cursor.operator = val;
                            setQueryGroup(updated);
                        }}
                        options={[{ label: 'AND', value: 'and' }, { label: 'OR', value: 'or' }]}
                        style={{ width: 80 }}
                    />
                    {group.rules.map((r, idx) =>
                        'field' in r
                            ? renderRule(r as Rule, groupPath, idx)
                            : renderGroup(r as RuleGroup, [...groupPath, idx])
                    )}
                    <Space>
                        <Button onClick={() => {
                            const updated = { ...queryGroup };
                            let cursor: any = updated;
                            for (const i of groupPath) cursor = cursor.rules[i];
                            const field = dataDictionary.fields[0]?.key;
                            const op = getOperators(getFieldType(field))[0];
                            cursor.rules.push({ field, operator: op, value: '' });
                            setQueryGroup(updated);
                        }}>+ Add Rule</Button>
                        <Button onClick={() => {
                            const updated = { ...queryGroup };
                            let cursor: any = updated;
                            for (const i of groupPath) cursor = cursor.rules[i];
                            cursor.rules.push({ operator: 'and', rules: [] });
                            setQueryGroup(updated);
                        }}>+ Add Group</Button>
                    </Space>
                </Space>
            </div>
        );
    };

    const transformValue = (val: any, type: string, operator: string): any => {
        if (operator === 'is' || operator === 'is not') return null;
        if (Array.isArray(val)) {
            return type === 'NUMBER'
                ? val.map(Number)
                : type === 'BOOLEAN'
                    ? val.map(v => v === 'true')
                    : val;
        }
        if (type === 'BOOLEAN') return val === 'true';
        if (type === 'NUMBER') return val.toString().includes('.') ? parseFloat(val) : parseInt(val);
        return val;
    };

    const transformToComposeQuery = (group: RuleGroup): any => ({
        operator: group.operator,
        items: group.rules.map(r =>
            'field' in r
                ? {
                    field: r.field,
                    operator: r.operator,
                    value: transformValue(r.value, getFieldType(r.field), r.operator),
                }
                : transformToComposeQuery(r as RuleGroup)
        ),
    });

    return (
        <Modal
            open={visible}
            title="Build Query"
            onCancel={onCancel}
            footer={[
                <Button key="cancel" onClick={onCancel}>Cancel</Button>,
                <Button key="confirm" type="primary" onClick={() => onConfirm(transformToComposeQuery(queryGroup))}>Enter</Button>,
            ]}
            width={900}
        >
            {renderGroup(queryGroup)}
        </Modal>
    );
};

export default QueryBuilder;
