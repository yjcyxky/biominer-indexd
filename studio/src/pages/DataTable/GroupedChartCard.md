# GroupedChartCard 组件使用文档

## 概述

`GroupedChartCard` 是一个支持两两变量分组统计与可视化的React组件，专门用于生物医学数据分析。该组件支持多种图表类型，能够处理复杂的生物医学数据集。

## 功能特性

- **多种图表类型支持**：Bar、Box、Violin、Scatter、Histogram
- **智能图表推荐**：根据数据类型自动推荐合适的图表类型
- **详细统计信息**：提供均值、中位数、标准差等统计指标
- **响应式设计**：支持容器大小变化自动重绘
- **交互式操作**：支持图表类型切换、数据查看等
- **特殊分析支持**：支持分组KM分析等复杂场景

## 组件接口

### Props

```typescript
interface GroupedChartCardProps {
    fields: API.DataDictionaryField[];           // 字段定义数组（至少2个字段）
    data: API.DatasetDataResponse['records'];    // 数据记录数组
    selectedColumns?: string[];                  // 选中的列名
    groupByField: string;                        // 分组字段（X轴）
    onClose?: () => void;                        // 关闭回调
    className?: string;                          // CSS类名
    resize?: () => void;                         // 重绘回调
    total: number;                               // 总记录数
    allowChangeChartType?: boolean;              // 是否允许切换图表类型
}
```

### 字段定义

```typescript
interface API.DataDictionaryField {
    key: string;                    // 字段键名
    name: string;                   // 字段显示名称
    data_type: string;              // 数据类型（STRING/NUMBER/BOOLEAN）
    description: string;            // 字段描述
    notes: string;                  // 字段备注
    allowed_values: any;            // 允许的值
    order: number;                  // 排序
}
```

## 使用方法

### 基本使用

```tsx
import GroupedChartCard from './GroupedChartCard';

const MyComponent = () => {
    const fields = [
        {
            key: 'cancer_type',
            name: 'Cancer Type',
            data_type: 'STRING',
            description: 'Type of cancer',
            notes: 'Categorical variable',
            allowed_values: ['Breast', 'Lung', 'Colon'],
            order: 1
        },
        {
            key: 'age',
            name: 'Age',
            data_type: 'NUMBER',
            description: 'Patient age',
            notes: 'Continuous variable',
            allowed_values: [],
            order: 2
        }
    ];

    const data = [
        { sample_id: '1', cancer_type: 'Breast', age: 45 },
        { sample_id: '2', cancer_type: 'Lung', age: 68 },
        // ... 更多数据
    ];

    return (
        <GroupedChartCard
            fields={fields}
            data={data}
            groupByField="cancer_type"
            total={data.length}
            allowChangeChartType={true}
            onClose={() => console.log('Chart closed')}
        />
    );
};
```

### 在现有项目中使用

```tsx
// 在DataTable页面中使用
import GroupedChartCard from './GroupedChartCard';

const DataTable = () => {
    const [selectedFields, setSelectedFields] = useState<API.DataDictionaryField[]>([]);
    const [groupByField, setGroupByField] = useState<string>('');
    
    const handleCreateGroupedChart = (fields: API.DataDictionaryField[], groupBy: string) => {
        if (fields.length >= 2) {
            setSelectedFields(fields);
            setGroupByField(groupBy);
        }
    };

    return (
        <div>
            {/* 其他组件 */}
            
            {selectedFields.length >= 2 && groupByField && (
                <GroupedChartCard
                    fields={selectedFields}
                    data={datasetData.records}
                    groupByField={groupByField}
                    selectedColumns={selectedFields.map(f => f.key)}
                    total={datasetData.total}
                    allowChangeChartType={true}
                    onClose={() => {
                        setSelectedFields([]);
                        setGroupByField('');
                    }}
                />
            )}
        </div>
    );
};
```

## 图表类型说明

### 1. Bar Chart (柱状图)
- **适用场景**：比较不同组的数值
- **数据要求**：
  - X轴为分类变量，Y轴为数值变量：显示各组的均值
  - X轴为分类变量，Y轴为分类变量：显示各组的计数
- **特点**：直观显示各组间的差异，支持颜色分组

### 2. Box Plot (箱线图)
- **适用场景**：显示数据分布和异常值
- **数据要求**：X轴为分类变量，Y轴为数值变量
- **特点**：显示中位数、四分位数、异常值，支持颜色分组

### 3. Violin Plot (小提琴图)
- **适用场景**：显示数据密度分布
- **数据要求**：X轴为分类变量，Y轴为数值变量
- **特点**：结合了箱线图和密度图的特点，支持颜色分组

### 4. Scatter Plot (散点图)
- **适用场景**：显示两个数值变量间的关系
- **数据要求**：X轴和Y轴都为数值变量
- **特点**：显示相关性、聚类等模式

### 5. Histogram (直方图)
- **适用场景**：显示单个数值变量的分布
- **数据要求**：Y轴为数值变量
- **特点**：显示数据分布形状

## 统计信息

组件提供详细的统计信息，包括：

- **Count**: 每组记录数
- **Mean**: 均值
- **Median**: 中位数
- **Min**: 最小值
- **Max**: 最大值
- **Std**: 标准差
- **Freq (%)**: 频率百分比

## 特殊分析场景

### 分组KM分析

对于生存分析等特殊场景，组件支持多字段分析：

```tsx
const kmFields = [
    { key: 'cancer_type', name: 'Cancer Type', data_type: 'STRING' },
    { key: 'os_months', name: 'OS Months', data_type: 'NUMBER' },
    { key: 'os_status', name: 'OS Status', data_type: 'STRING' }
];

// 数据中包含所有必要字段
const kmData = [
    { 
        sample_id: '1', 
        cancer_type: 'Breast', 
        os_months: 60, 
        os_status: 'Alive',
        // 其他字段...
    }
];
```

## 样式定制

组件使用Less样式，可以通过以下方式定制：

```less
.grouped-chart-card {
    // 自定义样式
    .ant-card {
        border-color: #your-color;
    }
    
    .chart-drag-handle {
        background-color: #your-color;
    }
}
```

## 注意事项

1. **字段数量**：必须提供至少2个字段
2. **分组字段**：必须指定一个有效的分组字段
3. **数据类型**：确保字段的data_type正确设置
4. **数据质量**：组件会自动过滤无效数据（null、undefined、空字符串）
5. **性能考虑**：大数据集时建议使用虚拟化或分页
6. **浏览器兼容性**：需要支持ES6+的现代浏览器

## 错误处理

组件内置了多种错误处理机制：

- 字段数量不足时显示提示信息
- 分组字段未指定时显示提示信息
- 数据为空时显示空状态
- 不支持的图表类型时显示错误信息
- 数据预处理失败时的降级处理

## 扩展开发

如需添加新的图表类型，可以：

1. 在`GroupedChartType`中添加新类型
2. 在`groupedChartTypeOptions`中添加选项
3. 在`renderVisualization`中添加渲染逻辑
4. 在`getRecommendedGroupedChartType`中添加推荐逻辑 