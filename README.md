# A distributed map reduce implementation in Rust
### Based on MapReduce: Simplified Data Processing on Large Clusters
### Based on MIT Course 6.5840: Distributed Systems

### Technologies 
Tonic & Tokio for Async Remote Procedure Call framework  
Everything else written in standard Rust  

### Features
- Multi Process Wokers (currently in single machine)  

### Usage
- Working on it

### Todo  
* Coordinator Shutdown after all task complete  
* Create Test
  * Correct Output
  * Correct Indexing
  * Map Parallelism
  * Reduce Parallelism
  * Job Count
  * Early Exit
  * Crash
  * Race Condition
* Remove Redundant Mutex
