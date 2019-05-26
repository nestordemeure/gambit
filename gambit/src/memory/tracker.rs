use systemstat::{Platform, System};

/// a struct to monitor memory use
pub struct MemoryTracker
{
   system: systemstat::platform::PlatformImpl,
   memory_at_creation: usize
}

/// returns the memory use in bytes
fn memory_usage<P>(system: &P) -> usize
   where P: Platform
{
   match system.memory()
   {
      Ok(mem) => (mem.total - mem.free).as_usize(),
      Err(x) => panic!("Unable to measure memory: {}", x)
   }
}

impl MemoryTracker
{
   /// builds a new memory tracker
   pub fn new() -> MemoryTracker
   {
      let system = System::new();
      let memory_at_creation = memory_usage(&system);
      MemoryTracker { system, memory_at_creation }
   }

   /// indicates the maount of free memory in Mo
   pub fn free_memory(&self) -> usize
   {
      match self.system.memory()
      {
         Ok(mem) => mem.free.as_usize() / 1_000_000,
         Err(x) => panic!("Unable to measure memory: {}", x)
      }
   }

   /// returns the current memory usage in bytes
   pub fn memory_usage(&self) -> usize
   {
      let current_memory = memory_usage(&self.system);
      current_memory - self.memory_at_creation
   }

   /// displays the current memory use, in Mo
   pub fn print_memory_usage(&self)
   {
      println!("memory consumption: {} Mo, free memory: {} Mo", self.memory_usage() / 1_000_000, self.free_memory());
   }
}

/*
   we can measure memory use at regular intervals to stop consumming it when we are a few hundreds of Mo before the end of the RAM
   it does not matter wether we are the one using the memory we just want to avoid crashing the computeur

   let sys = System::new();
   match sys.memory()
   {
      Ok(mem) => println!("\nMemory: {} used / {} ({} bytes)",
                          mem.total - mem.free,
                          mem.total,
                          (mem.total - mem.free).as_usize()),
      Err(x) => println!("\nMemory: error: {}", x)
   }
   // 1Go = 1000000000 bytes
*/