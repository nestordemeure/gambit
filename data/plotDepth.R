
library(ggplot2)
library(magrittr)
library(dplyr)

# import data from a given file
# NOTE: median makes plateaux that are hard to discern, the mean is sometimes way of the 95% zone
#file <- "/home/nestor/Documents/programming/F/symbolicRegression/data/kepler/ThompsonDepth.csv"
importData <- function(file)
{
   data <- read.csv(file)
   colnames(data) <- c("depth", "reward")
   data$name <- file %>% basename() %>% tools::file_path_sans_ext()
   return(data)
}

# displays an example
# exampleFolder <- "/home/nestor/Documents/programming/F/symbolicRegression/data/kepler"
displayExample <- function(exampleFolder)
{
   example <- basename(exampleFolder)
   files <- list.files(path=exampleFolder, pattern="*Depth.csv", full.names=TRUE) # gets only files relativ to depth
   
   X11()
   plot <- ggplot() + ggtitle(example) #+ xlim(-2, 2) #+ scale_y_continuous(trans='log2')
   #plot <- ggplot() + ggtitle(example)  #+ xlim(-2, 2)
   for(file in files)
   {
      data <- importData(file)
      plot <- plot + geom_point(data=data, aes(x=reward, y=depth, color=name, alpha=0.00001), size=1)
      #plot <- plot + geom_density(data=data, aes(x=reward, color=name, fill=name, alpha=0.1))
      #plot <- plot + geom_density_2d(data=data, aes(x=reward, y=depth, color=name, fill=name, alpha=0.1))
   }
   print(plot)
}

folder <- "/home/nestor/Documents/programming/F/symbolicRegression/data"
examples <- list.dirs(path=folder, full.names=TRUE, recursive = FALSE)
sapply(examples, displayExample)