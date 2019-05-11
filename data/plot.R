
#install.packages("ggplot2")
library(ggplot2)
library(magrittr)
library(dplyr)

# geometric mean (secured against 0)
gmean <- function(x, na.rm=TRUE)
{
   x <- x[x!=0]
   exp(sum(log(x), na.rm=na.rm) / length(x))
}

# import data from a given file
# NOTE: median makes plateaux that are hard to discern, the mean is sometimes way of the 95% zone
#file <- "/home/nestor/Documents/programming/F/symbolicRegression/data/kepler/randomDepth.csv"
importData <- function(file)
{
   data <- read.csv(file)
   colnames(data) <- c("evaluation", "error")
   data$error <- abs(data$error)
   #data <- data[is.finite(rowSums(data)),]
   data <- data %>% dplyr::group_by(evaluation) %>% dplyr::summarize(errorMin=quantile(error, 0.05), 
                                                                     errorMax=quantile(error,0.95), 
                                                                     error=gmean(error))
   data$name <- file %>% basename() %>% tools::file_path_sans_ext()
   return(data)
}

# displays an example
# exampleFolder <- "/home/nestor/Documents/programming/F/symbolicRegression/data/kepler"
displayExample <- function(exampleFolder)
{
   example <- basename(exampleFolder)
   files <- list.files(path=exampleFolder, pattern="*.csv", full.names=TRUE)
   
   X11()
   plot <- ggplot() + ggtitle(example) + scale_y_continuous(trans='log2')
   for(file in files)
   {
      data <- importData(file)
      plot <- plot + geom_line(data=data, aes(x=evaluation, y=error, color=name), size=1) +
         geom_line(data=data, aes(x=evaluation, y=errorMin, color=name), size=0.4, linetype=2) +
         geom_line(data=data, aes(x=evaluation, y=errorMax, color=name), size=0.4, linetype=2)
   }
   plot <- plot + ylim(NA,15)
   print(plot)
}

# displays all examples in a given folder
folder <- "/home/nestor/Documents/programming/F/symbolicRegression/data"
examples <- list.dirs(path=folder, full.names=TRUE, recursive = FALSE)
sapply(examples, displayExample)
