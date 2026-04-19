#!/usr/bin/env python3
"""
待办事项管理工具
功能：添加、查看、完成、删除和更新待办事项
数据持久化存储到本地JSON文件
"""

import json
from datetime import datetime
from typing import List, Optional
import uuid


class TodoItem:
    """待办事项数据类"""
    
    def __init__(self, title: str, description: str = "", priority: int = 1):
        self.id = self._generate_id()
        self.title = title
        self.description = description
        self.priority = priority
        self.completed = False
        self.created_at = datetime.now().isoformat()
        self.completed_at: Optional[str] = None
    
    @staticmethod
    def _generate_id() -> str:
        """生成唯一的待办事项ID"""
        return str(uuid.uuid4())[:8]
    
    def complete(self) -> None:
        """标记待办事项为已完成"""
        self.completed = True
        self.completed_at = datetime.now().isoformat()
    
    def to_dict(self) -> dict:
        """转换为字典格式"""
        return {
            "id": self.id,
            "title": self.title,
            "description": self.description,
            "priority": self.priority,
            "completed": self.completed,
            "created_at": self.created_at,
            "completed_at": self.completed_at
        }
    
    @classmethod
    def from_dict(cls, data: dict) -> "TodoItem":
        """从字典创建待办事项实例"""
        item = cls(data.get("title", ""), data.get("description", ""), data.get("priority", 1))
        item.id = data.get("id", item.id)
        item.completed = data.get("completed", False)
        item.created_at = data.get("created_at", item.created_at)
        item.completed_at = data.get("completed_at")
        return item
    
    def __repr__(self) -> str:
        status = "✓" if self.completed else "✗"
        return f"[{status}] {self.id} - {self.title}"


class TodoManager:
    """待办事项管理器"""
    
    def __init__(self, storage_file: str = "todos.json"):
        self.storage_file = storage_file
        self.todos: List[TodoItem] = []
        self.load()
    
    def load(self) -> None:
        """从文件加载待办事项"""
        try:
            with open(self.storage_file, "r", encoding="utf-8") as f:
                data = json.load(f)
                self.todos = [TodoItem.from_dict(item) for item in data]
        except FileNotFoundError:
            self.todos = []
        except json.JSONDecodeError:
            self.todos = []
    
    def save(self) -> None:
        """保存待办事项到文件"""
        with open(self.storage_file, "w", encoding="utf-8") as f:
            json.dump([item.to_dict() for item in self.todos], f, indent=2, ensure_ascii=False)
    
    def add(self, title: str, description: str = "", priority: int = 1) -> TodoItem:
        """添加新的待办事项"""
        todo = TodoItem(title, description, priority)
        self.todos.append(todo)
        self.save()
        return todo
    
    def get(self, todo_id: str) -> Optional[TodoItem]:
        """获取指定ID的待办事项"""
        for todo in self.todos:
            if todo.id == todo_id:
                return todo
        return None
    
    def list_all(self) -> List[TodoItem]:
        """列出所有待办事项"""
        return self.todos
    
    def list_pending(self) -> List[TodoItem]:
        """列出所有未完成的待办事项"""
        return [todo for todo in self.todos if not todo.completed]
    
    def list_completed(self) -> List[TodoItem]:
        """列出所有已完成的待办事项"""
        return [todo for todo in self.todos if todo.completed]
    
    def complete(self, todo_id: str) -> bool:
        """标记待办事项为已完成"""
        todo = self.get(todo_id)
        if todo:
            todo.complete()
            self.save()
            return True
        return False
    
    def uncomplete(self, todo_id: str) -> bool:
        """标记待办事项为未完成"""
        todo = self.get(todo_id)
        if todo:
            todo.completed = False
            todo.completed_at = None
            self.save()
            return True
        return False
    
    def delete(self, todo_id: str) -> bool:
        """删除待办事项"""
        for i, todo in enumerate(self.todos):
            if todo.id == todo_id:
                del self.todos[i]
                self.save()
                return True
        return False
    
    def update(self, todo_id: str, title: Optional[str] = None, 
               description: Optional[str] = None, priority: Optional[int] = None) -> bool:
        """更新待办事项"""
        todo = self.get(todo_id)
        if todo:
            if title is not None:
                todo.title = title
            if description is not None:
                todo.description = description
            if priority is not None:
                todo.priority = priority
            self.save()
            return True
        return False
    
    def clear_completed(self) -> int:
        """清除所有已完成的待办事项，返回删除数量"""
        count = len([t for t in self.todos if t.completed])
        self.todos = [todo for todo in self.todos if not todo.completed]
        self.save()
        return count
    
    def clear_all(self) -> int:
        """清除所有待办事项，返回删除数量"""
        count = len(self.todos)
        self.todos = []
        self.save()
        return count


def print_menu():
    """打印菜单"""
    print("\n" + "=" * 40)
    print("         待办事项管理工具")
    print("=" * 40)
    print("  1. 添加待办事项")
    print("  2. 查看所有待办事项")
    print("  3. 查看待完成事项")
    print("  4. 查看已完成事项")
    print("  5. 标记为已完成")
    print("  6. 标记为未完成")
    print("  7. 删除待办事项")
    print("  8. 更新待办事项")
    print("  9. 清除所有已完成的")
    print("  10. 统计信息")
    print("  0. 退出")
    print("=" * 40)


def display_todos(todos: List[TodoItem], title: str = "待办事项列表"):
    """显示待办事项列表"""
    print(f"\n{title}:")
    print("-" * 40)
    if not todos:
        print("  (空)")
    else:
        for i, todo in enumerate(todos, 1):
            status = "✓" if todo.completed else "✗"
            priority_emoji = {1: "低", 2: "中", 3: "高"}.get(todo.priority, "低")
            print(f"  {i}. [{status}] {todo.title} (优先级: {priority_emoji})")
            if todo.description:
                print(f"     描述: {todo.description}")
            print(f"     ID: {todo.id}")


def main():
    """主函数"""
    manager = TodoManager()
    
    while True:
        print_menu()
        choice = input("请选择操作 (0-10): ").strip()
        
        if choice == "0":
            print("感谢使用，再见!")
            break
        
        elif choice == "1":
            title = input("请输入标题: ").strip()
            if not title:
                print("标题不能为空!")
                continue
            
            description = input("请输入描述（可选）: ").strip()
            priority = input("请输入优先级（1-低, 2-中, 3-高，默认1）: ").strip()
            priority = int(priority) if priority.isdigit() and 1 <= int(priority) <= 3 else 1
            
            todo = manager.add(title, description, priority)
            print(f"✓ 已添加待办事项 [{todo.id}] {todo.title}")
        
        elif choice == "2":
            todos = manager.list_all()
            display_todos(todos, "所有待办事项")
        
        elif choice == "3":
            todos = manager.list_pending()
            display_todos(todos, "待完成的待办事项")
        
        elif choice == "4":
            todos = manager.list_completed()
            display_todos(todos, "已完成的待办事项")
        
        elif choice == "5":
            todo_id = input("请输入待办事项ID: ").strip()
            if manager.complete(todo_id):
                print("✓ 已标记为已完成")
            else:
                print("✗ 未找到该待办事项")
        
        elif choice == "6":
            todo_id = input("请输入待办事项ID: ").strip()
            if manager.uncomplete(todo_id):
                print("✓ 已标记为未完成")
            else:
                print("✗ 未找到该待办事项")
        
        elif choice == "7":
            todo_id = input("请输入待办事项ID: ").strip()
            if manager.delete(todo_id):
                print("✓ 已删除")
            else:
                print("✗ 未找到该待办事项")
        
        elif choice == "8":
            todo_id = input("请输入待办事项ID: ").strip()
            if not manager.get(todo_id):
                print("✗ 未找到该待办事项")
                continue
            
            print("直接回车跳过该字段")
            title = input("新标题: ").strip()
            description = input("新描述: ").strip()
            priority_str = input("新优先级（1-低, 2-中, 3-高）: ").strip()
            
            update_kwargs = {}
            if title:
                update_kwargs["title"] = title
            if description:
                update_kwargs["description"] = description
            if priority_str and priority_str.isdigit() and 1 <= int(priority_str) <= 3:
                update_kwargs["priority"] = int(priority_str)
            
            if update_kwargs:
                if manager.update(todo_id, **update_kwargs):
                    print("✓ 已更新")
                else:
                    print("✗ 更新失败")
            else:
                print("未提供任何更新内容")
        
        elif choice == "9":
            confirm = input("确定要清除所有已完成的待办事项吗？(y/N): ").strip().lower()
            if confirm == "y":
                count = manager.clear_completed()
                print(f"✓ 已清除 {count} 个已完成的待办事项")
            else:
                print("已取消")
        
        elif choice == "10":
            total = len(manager.todos)
            completed = len([t for t in manager.todos if t.completed])
            pending = total - completed
            print(f"\n统计信息:")
            print(f"  总计: {total}")
            print(f"  已完成: {completed}")
            print(f"  待完成: {pending}")
            if total > 0:
                print(f"  完成率: {completed / total * 100:.1f}%")
        
        else:
            print("无效的选择，请重新选择")


if __name__ == "__main__":
    main()