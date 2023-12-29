

# 


# indexer 需要处理的事件:

- ![event](https://github.com/ItsFunny/indexer-sdk/blob/main/src/client/event.rs)
- 跨语言的时候,提供给其他语言的数据都是bytes数组,需要根据后缀suffix判断数据类型
- reorg & delta 信息需要通过