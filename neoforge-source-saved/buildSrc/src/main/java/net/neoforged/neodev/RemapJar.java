package net.neoforged.neodev;

import org.gradle.api.file.RegularFileProperty;
import org.gradle.api.tasks.InputFile;
import org.gradle.api.tasks.JavaExec;
import org.gradle.api.tasks.OutputFile;
import org.gradle.api.tasks.TaskAction;

import javax.inject.Inject;

abstract class RemapJar extends JavaExec {
    @InputFile
    public abstract RegularFileProperty getInputJar();

    @OutputFile
    public abstract RegularFileProperty getOutputJar();

    @InputFile
    public abstract RegularFileProperty getMappings();

    @Inject
    public RemapJar() {}

    @Override
    @TaskAction
    public void exec() {
        args("--input", getInputJar().get().getAsFile().getAbsolutePath(),
             "--output", getOutputJar().get().getAsFile().getAbsolutePath(),
             "--names", getMappings().get().getAsFile().getAbsolutePath());
        super.exec();
    }
}
