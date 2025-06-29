import React, { useState } from 'react';
import { Button, Space, Select, message } from 'antd';
import GroupedChartCard from './GroupedChartCard';

// 示例数据
const mockData: API.DatasetDataResponse['records'] = [
    { sample_id: '1', cancer_type: 'Breast', age: 45, tumor_size: 2.5, survival_months: 60, vital_status: 'Alive' },
    { sample_id: '2', cancer_type: 'Breast', age: 52, tumor_size: 3.2, survival_months: 48, vital_status: 'Alive' },
    { sample_id: '3', cancer_type: 'Lung', age: 68, tumor_size: 4.1, survival_months: 24, vital_status: 'Dead' },
    { sample_id: '4', cancer_type: 'Lung', age: 61, tumor_size: 2.8, survival_months: 36, vital_status: 'Alive' },
    { sample_id: '5', cancer_type: 'Colon', age: 58, tumor_size: 3.5, survival_months: 42, vital_status: 'Alive' },
    { sample_id: '6', cancer_type: 'Colon', age: 49, tumor_size: 2.1, survival_months: 72, vital_status: 'Alive' },
    { sample_id: '7', cancer_type: 'Breast', age: 55, tumor_size: 1.8, survival_months: 84, vital_status: 'Alive' },
    { sample_id: '8', cancer_type: 'Lung', age: 72, tumor_size: 5.2, survival_months: 12, vital_status: 'Dead' },
    { sample_id: '9', cancer_type: 'Colon', age: 63, tumor_size: 2.9, survival_months: 54, vital_status: 'Alive' },
    { sample_id: '10', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '11', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '12', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '13', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '14', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '15', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '16', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '17', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '18', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '19', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '20', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '21', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' },
    { sample_id: '22', cancer_type: 'Breast', age: 47, tumor_size: 2.3, survival_months: 66, vital_status: 'Alive' }
];

// 示例字段定义
const mockFields: API.DataDictionaryField[] = [
    {
        key: 'cancer_type',
        name: 'Cancer Type',
        data_type: 'STRING',
        description: 'Type of cancer',
        notes: 'Categorical variable for cancer types',
        allowed_values: ['Breast', 'Lung', 'Colon'],
        order: 1
    },
    {
        key: 'age',
        name: 'Age',
        data_type: 'NUMBER',
        description: 'Patient age at diagnosis',
        notes: 'Continuous variable for patient age',
        allowed_values: [],
        order: 2
    },
    {
        key: 'tumor_size',
        name: 'Tumor Size',
        data_type: 'NUMBER',
        description: 'Tumor size in cm',
        notes: 'Continuous variable for tumor size',
        allowed_values: [],
        order: 3
    },
    {
        key: 'survival_months',
        name: 'Survival Months',
        data_type: 'NUMBER',
        description: 'Survival time in months',
        notes: 'Continuous variable for survival time',
        allowed_values: [],
        order: 4
    },
    {
        key: 'vital_status',
        name: 'Vital Status',
        data_type: 'STRING',
        description: 'Patient vital status',
        notes: 'Categorical variable for vital status',
        allowed_values: ['Alive', 'Dead'],
        order: 5
    }
];

const GroupedChartCardExample: React.FC = () => {
    const [selectedFields, setSelectedFields] = useState<API.DataDictionaryField[]>([]);
    const [groupByField, setGroupByField] = useState<string>('');
    const [showChart, setShowChart] = useState<boolean>(false);

    const handleFieldSelection = (fieldKeys: string[]) => {
        const fields = mockFields.filter(field => fieldKeys.includes(field.key));
        setSelectedFields(fields);
        
        // 自动选择第一个字段作为分组字段
        if (fields.length > 0 && !groupByField) {
            setGroupByField(fields[0].key);
        }
    };

    const handleGroupByFieldChange = (fieldKey: string) => {
        setGroupByField(fieldKey);
    };

    const handleCreateChart = () => {
        if (selectedFields.length < 2) {
            message.error('请选择至少两个字段进行分组分析');
            return;
        }
        if (!groupByField) {
            message.error('请选择分组字段');
            return;
        }
        setShowChart(true);
    };

    const handleCloseChart = () => {
        setShowChart(false);
    };

    return (
        <div style={{ padding: '20px' }}>
            <h2>分组图表分析示例</h2>
            
            <Space direction="vertical" style={{ width: '100%' }}>
                <div>
                    <label>选择字段进行分组分析（至少选择2个字段）：</label>
                    <Select
                        mode="multiple"
                        style={{ width: '100%', marginTop: '8px' }}
                        placeholder="选择字段"
                        options={mockFields.map(field => ({
                            label: `${field.name} (${field.data_type})`,
                            value: field.key
                        }))}
                        onChange={handleFieldSelection}
                        maxTagCount={5}
                    />
                </div>

                {selectedFields.length >= 2 && (
                    <div>
                        <label>选择分组字段（X轴）：</label>
                        <Select
                            style={{ width: '100%', marginTop: '8px' }}
                            placeholder="选择分组字段"
                            value={groupByField}
                            onChange={handleGroupByFieldChange}
                            options={selectedFields.map(field => ({
                                label: `${field.name} (${field.data_type})`,
                                value: field.key
                            }))}
                        />
                    </div>
                )}

                <Button 
                    type="primary" 
                    onClick={handleCreateChart}
                    disabled={selectedFields.length < 2 || !groupByField}
                >
                    创建分组图表
                </Button>

                {showChart && selectedFields.length >= 2 && groupByField && (
                    <div style={{ height: '500px', border: '1px solid #d9d9d9', borderRadius: '6px' }}>
                        <GroupedChartCard
                            groupByField={groupByField}
                            fields={selectedFields}
                            data={mockData}
                            selectedColumns={selectedFields.map(f => f.key)}
                            onClose={handleCloseChart}
                            total={mockData.length}
                            allowChangeChartType={true}
                            resize={() => {
                                // 处理图表重绘
                                console.log('Chart resized');
                            }}
                        />
                    </div>
                )}

                <div style={{ marginTop: '20px' }}>
                    <h3>使用说明：</h3>
                    <ul>
                        <li>选择两个字段进行分组分析</li>
                        <li>指定一个字段作为分组字段（X轴）</li>
                        <li>支持多种图表类型：Bar、Box、Violin、Scatter、Histogram</li>
                        <li>Bar、Box、Violin适用于X是分组变量、Y是数值变量的情况</li>
                        <li>Scatter适用于两变量都是数值的情况</li>
                        <li>Histogram适用于Y轴是数值变量的情况</li>
                        <li>点击眼睛图标查看详细统计信息</li>
                        <li>可以通过下拉菜单切换图表类型</li>
                        <li>支持特殊分析场景，如分组KM分析（需要多个字段）</li>
                    </ul>
                </div>

                <div style={{ marginTop: '20px' }}>
                    <h3>图表类型说明：</h3>
                    <ul>
                        <li><strong>Bar Chart</strong>：比较不同组的数值，支持分类变量vs数值变量或分类变量vs分类变量</li>
                        <li><strong>Box Plot</strong>：显示数据分布和异常值，适用于分类变量vs数值变量</li>
                        <li><strong>Violin Plot</strong>：显示数据密度分布，适用于分类变量vs数值变量</li>
                        <li><strong>Scatter Plot</strong>：显示两个数值变量间的关系，适用于数值变量vs数值变量</li>
                        <li><strong>Histogram</strong>：显示单个数值变量的分布，适用于Y轴是数值变量</li>
                    </ul>
                </div>
            </Space>
        </div>
    );
};

export default GroupedChartCardExample; 