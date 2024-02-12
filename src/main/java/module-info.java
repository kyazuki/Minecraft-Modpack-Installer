module kyazuki {
    requires transitive java.logging;

    requires com.fasterxml.jackson.databind;
    requires com.fasterxml.jackson.dataformat.yaml;
    requires com.fasterxml.jackson.datatype.jsr310;
    requires spring.web;

    requires javafx.controls;
    requires transitive javafx.graphics;
    requires javafx.fxml;

    opens kyazuki to javafx.fxml;
    opens kyazuki.controller to javafx.fxml;

    exports kyazuki;
    exports kyazuki.dataclass;
}
