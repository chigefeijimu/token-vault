"""
工具模块 - 提供基础工具函数
"""

def health_check():
    """
    健康检查函数，用于验证服务是否正常运行
    
    Returns:
        dict: 包含健康状态的字典
    """
    return {
        "status": "healthy",
        "service": "main",
        "version": "1.0.0"
    }


def validate_input(data):
    """
    验证输入数据的有效性
    
    Args:
        data: 待验证的数据
        
    Returns:
        bool: 验证是否通过
    """
    if data is None:
        return False
    if isinstance(data, str) and len(data.strip()) == 0:
        return False
    return True


def process_data(data):
    """
    处理数据的主函数
    
    Args:
        data: 待处理的数据
        
    Returns:
        dict: 处理结果
    """
    if not validate_input(data):
        return {"error": "Invalid input data"}
    
    return {
        "success": True,
        "processed": True,
        "data": data
    }


class DataProcessor:
    """数据处理器类"""
    
    def __init__(self, config=None):
        """
        初始化数据处理器
        
        Args:
            config: 配置字典
        """
        self.config = config or {}
        self.initialized = True
    
    def process(self, data):
        """
        处理单个数据项
        
        Args:
            data: 待处理的数据
            
        Returns:
            处理后的数据
        """
        if not validate_input(data):
            return None
        
        # 这里添加具体的处理逻辑
        return {
            "input": data,
            "output": f"Processed: {data}",
            "processor": "DataProcessor"
        }
    
    def batch_process(self, data_list):
        """
        批量处理数据
        
        Args:
            data_list: 数据列表
            
        Returns:
            处理结果列表
        """
        results = []
        for item in data_list:
            result = self.process(item)
            if result:
                results.append(result)
        return results


if __name__ == "__main__":
    # 测试代码
    print("Health Check:", health_check())
    
    processor = DataProcessor()
    test_data = ["item1", "item2", "item3"]
    results = processor.batch_process(test_data)
    
    print(f"Processed {len(results)} items successfully")